#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


extern crate libc;
use std::ffi::CString;
use std::os::raw::c_char;

pub enum History {
  KeepLast {n: u32},
  KeepAll
}

pub enum Durability {
  Volatile,
  TransientLocal,
  Transient,
  Persistent
}

pub struct QoS {
  qos: *mut dds_qos_t
}

impl QoS {
  fn new() -> QoS {
    QoS { qos : unsafe {dds_create_qos()} }
  }

  fn reset(&mut self) {
    unsafe { dds_qos_reset(self.qos) }
  }

  fn history(&mut self, h: &History) {
    match h  {
      History::KeepLast { n } => {
        unsafe { dds_qset_history(self.qos, dds_history_kind_DDS_HISTORY_KEEP_LAST, *n as i32)}
      },
      History::KeepAll => {
        unsafe { dds_qset_history(self.qos, dds_history_kind_DDS_HISTORY_KEEP_ALL, 0)}
      }
    }
  }

  fn partitions(&mut self, ps: &[String]) {        
    // let mut xs : [*const c_char; ps.len()] = [ std::ptr::null(); ps.len()]; 
    // let p = CString::new(ps[0]).unwrap().as_ptr();
    let mut cps : Vec<*const c_char> = ps.iter().map(|s| CString::new(String::from(s)).unwrap().as_ptr()).collect();    
    unsafe { dds_qset_partition(self.qos, ps.len() as u32, cps.as_mut_ptr() as  *mut *const ::std::os::raw::c_char) }  
    
  }


}

impl PartialEq for QoS {
  fn eq(&self, other: &Self) -> bool {
    unsafe { dds_qos_equal(self.qos, other.qos) }
  }
}

impl Eq for QoS {}

impl Clone for QoS {
  fn clone(&self) -> Self {
    let dst = QoS { qos : unsafe { dds_create_qos()} };
    unsafe { dds_copy_qos(dst.qos, self.qos as *const dds_qos_t) };  
    dst 
  }
}

impl Drop for QoS {
  fn drop(&mut self) {
    unsafe { dds_qos_delete(self.qos) };
  }
}


pub struct Participant {
  entity: dds_entity_t
}

impl Drop for Participant {
  fn drop(&mut self) {
    unsafe { dds_delete(self.entity) };
  }
}

impl Participant {
  pub fn new(d: dds_domainid_t) -> Participant {
    let e = unsafe { dds_create_participant(d, std::ptr::null(), std::ptr::null()) };
    Participant { entity: e }
  }  
}
