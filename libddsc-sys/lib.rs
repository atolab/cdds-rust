#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#[allow(improper_ctypes)]


pub const DDS_MIN_PSEUDO_HANDLE : dds_entity_t = 0x7fff0000 as dds_entity_t;

/* @defgroup builtintopic_constants Convenience constants for referring to builtin topics
 *
 * These constants can be used in place of an actual dds_topic_t, when creating
 * readers or writers for builtin-topics.
 *
 * @{
 */
pub const DDS_BUILTIN_TOPIC_DCPSPARTICIPANT     : dds_entity_t = (DDS_MIN_PSEUDO_HANDLE + 1) as dds_entity_t;
pub const DDS_BUILTIN_TOPIC_DCPSTOPIC           : dds_entity_t = (DDS_MIN_PSEUDO_HANDLE + 2) as dds_entity_t;
pub const DDS_BUILTIN_TOPIC_DCPSPUBLICATION     : dds_entity_t = (DDS_MIN_PSEUDO_HANDLE + 3) as dds_entity_t;
pub const DDS_BUILTIN_TOPIC_DCPSSUBSCRIPTION    : dds_entity_t = (DDS_MIN_PSEUDO_HANDLE + 4) as dds_entity_t;


/** Special handle representing the entity corresponding to the CycloneDDS library itself */
pub const DDS_CYCLONEDDS_HANDLE                 : dds_entity_t = (DDS_MIN_PSEUDO_HANDLE + 256) as dds_entity_t;

pub const DDS_DOMAIN_DEFAULT                : u32 = 0xffffffff as u32;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

