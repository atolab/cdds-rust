use std::mem::MaybeUninit;
use std::{thread, time};
use std::ffi::CStr;
use libddsc_sys::*;

const MAX_SAMPLES : usize = 32;

unsafe extern "C" fn on_data(dr: dds_entity_t, _arg: *mut std::os::raw::c_void) {
    let mut si: [dds_sample_info_t; MAX_SAMPLES] = { MaybeUninit::uninit().assume_init() };
    let mut samples : [*mut ::std::os::raw::c_void; MAX_SAMPLES] = [std::ptr::null_mut(); MAX_SAMPLES as usize];
    samples[0] = std::ptr::null_mut();

    let n = dds_take(
        dr,
        samples.as_mut_ptr() as *mut *mut libc::c_void,
        si.as_mut_ptr() as *mut dds_sample_info_t,
        MAX_SAMPLES as u64,
        MAX_SAMPLES as u32
    );
    for i in 0..n {
        if si[i as usize].valid_data {
            let sample= samples[i as usize] as *mut dds_builtintopic_endpoint_t;
            print!("Discovered Pub for: {}", CStr::from_ptr((*sample).topic_name).to_str().unwrap());
            let mut n = 0u32;
            let mut ps: *mut *mut ::std::os::raw::c_char = std::ptr::null_mut();
            let has_partition = dds_qget_partition((*sample).qos, &mut n as *mut u32, &mut ps as *mut *mut *mut ::std::os::raw::c_char);
            if has_partition {
                for k in 0..n {
                    let p = CStr::from_ptr(*(ps.offset(k as isize)));
                    print!("Partition: {}", p.to_str().unwrap());
                }
            }
        }
    }
    print!("Read {} samples\n", n);
    dds_return_loan(dr, samples.as_mut_ptr() as *mut *mut libc::c_void, MAX_SAMPLES as i32);
}
fn main() {
    unsafe {
        let dp = dds_create_participant(DDS_DOMAIN_DEFAULT, std::ptr::null(), std::ptr::null());
        let mut publication = Box::new(String::from("Publication"));
        let mut subscription = Box::new(String::from("Subscription"));
        let pub_listener = dds_create_listener(publication.as_mut_ptr() as *mut std::os::raw::c_void);
        dds_lset_data_available(pub_listener, Some(on_data));

        let _pr = dds_create_reader(dp, DDS_BUILTIN_TOPIC_DCPSPUBLICATION, std::ptr::null(), pub_listener);

        let sub_listener = dds_create_listener(subscription.as_mut_ptr() as *mut std::os::raw::c_void);
        dds_lset_data_available(sub_listener, Some(on_data));
        let _sr = dds_create_reader(dp, DDS_BUILTIN_TOPIC_DCPSSUBSCRIPTION, std::ptr::null(), sub_listener);
        loop {
            let duration = time::Duration::from_millis(2000);
            thread::sleep(duration);
        }

    }
}