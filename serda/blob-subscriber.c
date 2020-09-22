/*
 * Copyright(c) 2020 ADLINK Technology Limited and others
 *
 * This program and the accompanying materials are made available under the
 * terms of the Eclipse Public License v. 2.0 which is available at
 * http://www.eclipse.org/legal/epl-2.0, or the Eclipse Distribution License
 * v. 1.0 which is available at
 * http://www.eclipse.org/org/documents/edl-v10.php.
 *
 * SPDX-License-Identifier: EPL-2.0 OR BSD-3-Clause
 */
#include <assert.h>
#include <limits.h>
#include <string.h>

#include "dds/ddsrt/mh3.h"
#include "dds/ddsrt/md5.h"
#include "dds/ddsrt/bswap.h"

#include "dds/dds.h"
#include "dds/ddsi/ddsi_serdata.h"
#include "dds/ddsi/q_radmin.h"

#include "dds/dds.h"
#include "dds/ddsrt/atomics.h"
#include "dds/ddsrt/process.h"
#include "dds/ddsrt/threads.h"

struct z_ddsi_payload {
  struct ddsi_serdata sd;
  size_t size;
  enum ddsi_serdata_kind kind;
  unsigned char* payload;
};


static bool z_sertopic_equal (const struct ddsi_sertopic *acmn, const struct ddsi_sertopic *bcmn)
{
  // no fields in stp beyond the common ones, and those are all checked for equality before this function is called
  (void) acmn; (void) bcmn;
  return true;
}

static uint32_t z_sertopic_hash (const struct ddsi_sertopic *tpcmn)
{
  // nothing beyond the common fields
  (void) tpcmn;
  return 0;
}


static void z_sertopic_free(struct ddsi_sertopic * tpcmn) {
  ddsi_sertopic_fini(tpcmn);
}


static void z_sertopic_zero_samples(const struct ddsi_sertopic * d, void * samples, size_t count) {
  (void)d;
  (void)samples;
  (void)count;
  /* Not using code paths that rely on the samples getting zero'd out */
}


static void z_sertopic_realloc_samples(
  void ** ptrs, const struct ddsi_sertopic * d,
  void * old, size_t oldcount, size_t count)
{
  (void)(ptrs);
  (void)(d);
  (void)(old);
  (void)(oldcount);
  (void)(count);
  /* Not using code paths that rely on this (loans, dispose, unregister with instance handle,
     content filters) */
  abort();
}

static void z_sertopic_free_samples(
  const struct ddsi_sertopic * d, void ** ptrs, size_t count,
  dds_free_op_t op)
{
  (void)(d);    // unused
  (void)(ptrs);    // unused
  (void)(count);    // unused
  /* Not using code paths that rely on this (dispose, unregister with instance handle, content
     filters) */
  assert(!(op & DDS_FREE_ALL_BIT));
  (void) op;
}

static const struct ddsi_sertopic_ops z_sertopic_ops = {
  .free = z_sertopic_free,
  .zero_samples = z_sertopic_zero_samples,
  .realloc_samples = z_sertopic_realloc_samples,
  .free_samples = z_sertopic_free_samples,
  .equal = z_sertopic_equal,
  .hash = z_sertopic_hash
};

static bool z_serdata_eqkey(const struct ddsi_serdata * a, const struct ddsi_serdata * b)
{
  (void)(a);
  (void)(b);
  /* ROS 2 doesn't do keys in a meaningful way yet */
  printf("Called <z_serdata_eqkey>\n");
  return true;
}

static uint32_t z_serdata_size(const struct ddsi_serdata * sd)
{
  printf("Called <z_serdata_size>\n");
  struct z_ddsi_payload * zp = (struct z_ddsi_payload *)sd;
  return zp->size;
}

static void z_serdata_free(struct ddsi_serdata * sd)
{
  printf("Called <z_serdata_free>\n");
  struct z_ddsi_payload * zp = (struct z_ddsi_payload *)sd;
  free(zp->payload);
  zp->size = 0;
  // TODO: verify that dds_fini does not need to be called on zp->sd
  free(zp);
}

static struct ddsi_serdata *z_serdata_from_ser_iov (const struct ddsi_sertopic *tpcmn, enum ddsi_serdata_kind kind, ddsrt_msg_iovlen_t niov, const ddsrt_iovec_t *iov, size_t size)
{
  printf("==> <z_serdata_from_ser_iov> for %s -- size %zu\n", tpcmn->name, size);
  struct z_ddsi_payload *zp = (struct z_ddsi_payload *)malloc(sizeof(struct z_ddsi_payload));
  ddsi_serdata_init(&zp->sd, tpcmn, kind);
  zp->size = size;
  zp->kind = kind;
  zp->payload = 0;
  switch (kind) {
    case SDK_EMPTY:
      break;
    case SDK_DATA:
      zp->payload = malloc(size);
      int offset = 0;
      int csize = 0;
      for (int i = 0; i < niov; ++i) {
        csize += iov[i].iov_len;
        assert(csize <= size);
        memcpy(zp->payload + offset, iov[i].iov_base, iov[i].iov_len);
        offset += iov[i].iov_len;
      }
      break;
    case SDK_KEY:
      break;
  }
  return (struct ddsi_serdata *)zp;
}

static struct ddsi_serdata *z_serdata_from_ser (const struct ddsi_sertopic *tpcmn, enum ddsi_serdata_kind kind, const struct nn_rdata *fragchain, size_t size)
{
  printf("Called <z_serdata_from_ser> for %s\n", tpcmn->name);
  // This currently assumes that there is only one fragment.
  assert (fragchain->nextfrag == NULL);
  ddsrt_iovec_t iov = {
    .iov_base = NN_RMSG_PAYLOADOFF (fragchain->rmsg, NN_RDATA_PAYLOAD_OFF (fragchain)),
    .iov_len = fragchain->maxp1 // fragchain->min = 0 for first fragment, by definition
  };
  return z_serdata_from_ser_iov (tpcmn, kind, 1, &iov, size);
}

static struct ddsi_serdata *z_serdata_to_topicless (const struct ddsi_serdata *sd) {
  return ddsi_serdata_ref(sd);
}


static const struct ddsi_serdata_ops z_serdata_ops = {
  .get_size = z_serdata_size,
  .eqkey = z_serdata_eqkey,
  .from_ser = z_serdata_from_ser,
  .from_ser_iov = z_serdata_from_ser_iov,
  .to_topicless = z_serdata_to_topicless
};


int main(int argc, char *argv[])
{
  char* partition = NULL;

  if (argc < 3) {
    printf("USAGE:\n\tsub <topic-name> <type_name> [<partition>]\n");
    exit(1);
  }
  if (argc > 3) {
    partition = argv[3];
  }

  dds_return_t rc;

  const dds_entity_t pp = dds_create_participant (DDS_DOMAIN_DEFAULT, NULL, NULL);
  printf("Topic Name: %s\n", argv[1]);

  struct ddsi_sertopic *st = (struct ddsi_sertopic*) malloc(sizeof(struct ddsi_sertopic));
  printf("Initialising SerTopic\n");
  ddsi_sertopic_init (st, argv[1], argv[2], &z_sertopic_ops, &z_serdata_ops, true);

  printf("Creating Topic\n");
  const dds_entity_t tp = dds_create_topic_generic (pp, &st, NULL, NULL, NULL);

  dds_qos_t *qos = NULL;
  if (partition != NULL) {
  printf("Creating Qos\n");
    qos = dds_create_qos();
    dds_qset_partition1(qos, partition);
  }
  printf("Creating Reader\n");
  const dds_entity_t rd = dds_create_reader (pp, tp, qos, NULL);

  do
  {
    struct ddsi_serdata *sample;
    dds_sample_info_t si;
    rc = dds_takecdr(rd, &sample, 1, &si, DDS_ANY_STATE);
    printf("Received %d samples\n", rc);
    if (rc > 0) {
      struct z_ddsi_payload *zp = (struct z_ddsi_payload *)sample;
      printf("Payload:\n Size = %d\n", (int)zp->size);
      for (int i = 0; i < zp->size; ++i) {
        printf("%d ", (int)zp->payload[i]);
      }
      printf("\n");
    }
    dds_sleepfor (DDS_MSECS (1000));
  } while(true);

  rc = dds_delete (pp);
}
