use std::mem::MaybeUninit;
use libddsc_sys::*;
use std::ffi::{CStr, CString};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

const MAX_SAMPLES: usize = 32;

#[derive(Debug)]
pub enum MatchedEntity {
    DiscoveredPublication {
        topic_name: String,
        type_name: String,
        partition: Option<String>,
        qos: Arc<*mut dds_qos_t>
    },
    UndiscoveredPublication {
        topic_name: String,
        type_name: String,
        partition: Option<String>,
    },
    DiscoveredSubscription {
        topic_name: String,
        type_name: String,
        partition: Option<String>,
        qos: Arc<*mut dds_qos_t>
    },
    UndiscoveredSubscription {
        topic_name: String,
        type_name: String,
        partition: Option<String>,
    },
}

unsafe extern "C" fn on_data(dr: dds_entity_t, arg: *mut std::os::raw::c_void) {
    let btx = Box::from_raw(arg as *mut (bool, Sender<MatchedEntity>));

    #[allow(clippy::uninit_assumed_init)]
    let mut si: [dds_sample_info_t; MAX_SAMPLES] = { MaybeUninit::uninit().assume_init() };
    let mut samples: [*mut ::std::os::raw::c_void; MAX_SAMPLES] =
        [std::ptr::null_mut(); MAX_SAMPLES as usize];
    samples[0] = std::ptr::null_mut();

    let n = dds_take(
        dr,
        samples.as_mut_ptr() as *mut *mut libc::c_void,
        si.as_mut_ptr() as *mut dds_sample_info_t,
        MAX_SAMPLES as u64,
        MAX_SAMPLES as u32,
    );
    for i in 0..n {
        if si[i as usize].valid_data {
            let sample = samples[i as usize] as *mut dds_builtintopic_endpoint_t;
            let topic_name = CStr::from_ptr((*sample).topic_name).to_str().unwrap();
            let type_name = CStr::from_ptr((*sample).type_name).to_str().unwrap();
            let mut n = 0u32;
            let mut ps: *mut *mut ::std::os::raw::c_char = std::ptr::null_mut();
            let qos = dds_create_qos();
            dds_copy_qos(qos, (*sample).qos);
            let bqos = Arc::new(qos);

            let _ = dds_qget_partition(
                (*sample).qos,
                &mut n as *mut u32,
                &mut ps as *mut *mut *mut ::std::os::raw::c_char,
            );
            if n > 0 {
                for k in 0..n {
                    let p = CStr::from_ptr(*(ps.offset(k as isize))).to_str().unwrap();
                    if si[i as usize].instance_state == dds_instance_state_DDS_IST_ALIVE {
                        if btx.0 {
                            (btx.1)
                                .send(MatchedEntity::DiscoveredPublication {
                                    topic_name: String::from(topic_name),
                                    type_name: String::from(type_name),
                                    partition: Some(String::from(p)),
                                    qos: bqos.clone()
                                })
                                .unwrap();
                        } else {
                            (btx.1)
                                .send(MatchedEntity::DiscoveredSubscription {
                                    topic_name: String::from(topic_name),
                                    type_name: String::from(type_name),
                                    partition: Some(String::from(p)),
                                    qos: bqos.clone()
                                })
                                .unwrap();
                        }
                    } else if btx.0 {
                        (btx.1)
                            .send(MatchedEntity::UndiscoveredPublication {
                                topic_name: String::from(topic_name),
                                type_name: String::from(type_name),
                                partition: Some(String::from(p)),
                            })
                            .unwrap();
                    } else {
                        (btx.1)
                            .send(MatchedEntity::UndiscoveredSubscription {
                                topic_name: String::from(topic_name),
                                type_name: String::from(type_name),
                                partition: Some(String::from(p)),
                            })
                            .unwrap();
                    }
                }
            } else if si[i as usize].instance_state == dds_instance_state_DDS_IST_ALIVE {
                if btx.0 {
                    (btx.1)
                        .send(MatchedEntity::DiscoveredPublication {
                            topic_name: String::from(topic_name),
                            type_name: String::from(type_name),
                            partition: None,
                            qos: bqos.clone()
                        })
                        .unwrap();
                } else {
                    (btx.1)
                        .send(MatchedEntity::DiscoveredSubscription {
                            topic_name: String::from(topic_name),
                            type_name: String::from(type_name),
                            partition: None,
                            qos: bqos.clone()
                        })
                        .unwrap();
                }
            } else if btx.0 {
                (btx.1)
                    .send(MatchedEntity::UndiscoveredPublication {
                        topic_name: String::from(topic_name),
                        type_name: String::from(type_name),
                        partition: None,
                    })
                    .unwrap();
            } else {
                (btx.1)
                    .send(MatchedEntity::UndiscoveredSubscription {
                        topic_name: String::from(topic_name),
                        type_name: String::from(type_name),
                        partition: None,
                    })
                    .unwrap();
            }
        }
    }
    dds_return_loan(
        dr,
        samples.as_mut_ptr() as *mut *mut libc::c_void,
        MAX_SAMPLES as i32,
    );
    Box::into_raw(btx);
}
fn main() {
    unsafe {
        let (tx, rx): (Sender<MatchedEntity>, Receiver<MatchedEntity>) = channel();
        let ptx = Box::new((true, tx.clone()));
        let stx = Box::new((false, tx));
        let dp = dds_create_participant(DDS_DOMAIN_DEFAULT, std::ptr::null(), std::ptr::null());
        let _ = cdds_create_blob_sertopic(dp, CString::new("Foo").unwrap().into_raw(), CString::new("Bar").unwrap().into_raw(), true);
        let pub_listener = dds_create_listener(Box::into_raw(ptx) as *mut std::os::raw::c_void);
        dds_lset_data_available(pub_listener, Some(on_data));

        let _pr = dds_create_reader(
            dp,
            DDS_BUILTIN_TOPIC_DCPSPUBLICATION,
            std::ptr::null(),
            pub_listener,
        );

        let sub_listener = dds_create_listener(Box::into_raw(stx) as *mut std::os::raw::c_void);
        dds_lset_data_available(sub_listener, Some(on_data));
        let _sr = dds_create_reader(
            dp,
            DDS_BUILTIN_TOPIC_DCPSSUBSCRIPTION,
            std::ptr::null(),
            sub_listener,
        );
        while let Ok(me) = rx.recv() {
            println!("{:?}", me);
        }
    }
}
