extern crate libc;
use std::ffi::CString;
use std::os::raw::c_char;

extern crate libddsc_sys;

pub enum History {
    KeepLast { n: u32 },
    KeepAll,
}

pub enum Durability {
    Volatile,
    TransientLocal,
    Transient,
    Persistent,
}

pub struct QoS {
    qos: *mut libddsc_sys::dds_qos_t,
}

impl QoS {
    pub fn reset(&mut self) {
        unsafe { libddsc_sys::dds_qos_reset(self.qos) }
    }

    pub fn history(&mut self, h: &History) {
        match h {
            History::KeepLast { n } => unsafe {
                libddsc_sys::dds_qset_history(
                    self.qos,
                    libddsc_sys::dds_history_kind_DDS_HISTORY_KEEP_LAST,
                    *n as i32,
                )
            },
            History::KeepAll => unsafe {
                libddsc_sys::dds_qset_history(
                    self.qos,
                    libddsc_sys::dds_history_kind_DDS_HISTORY_KEEP_ALL,
                    0,
                )
            },
        }
    }

    pub fn partitions(&mut self, ps: &[String]) {
        // let mut xs : [*const c_char; ps.len()] = [ std::ptr::null(); ps.len()];
        // let p = CString::new(ps[0]).unwrap().as_ptr();
        let mut cps: Vec<*const c_char> = ps
            .iter()
            .map(|s| CString::new(String::from(s)).unwrap().into_raw() as *const c_char)
            .collect();
        unsafe {
            libddsc_sys::dds_qset_partition(
                self.qos,
                ps.len() as u32,
                cps.as_mut_ptr() as *mut *const ::std::os::raw::c_char,
            )
        }
    }
}

impl Default for QoS {
    fn default() -> QoS {
        QoS {
            qos: unsafe { libddsc_sys::dds_create_qos() },
        }
    }
}

impl PartialEq for QoS {
    fn eq(&self, other: &Self) -> bool {
        unsafe { libddsc_sys::dds_qos_equal(self.qos, other.qos) }
    }
}

impl Eq for QoS {}

impl Clone for QoS {
    fn clone(&self) -> Self {
        let dst = QoS {
            qos: unsafe { libddsc_sys::dds_create_qos() },
        };
        unsafe { libddsc_sys::dds_copy_qos(dst.qos, self.qos as *const libddsc_sys::dds_qos_t) };
        dst
    }
}

impl Drop for QoS {
    fn drop(&mut self) {
        unsafe { libddsc_sys::dds_qos_delete(self.qos) };
    }
}

pub struct Participant {
    entity: libddsc_sys::dds_entity_t,
}

impl Drop for Participant {
    fn drop(&mut self) {
        unsafe { libddsc_sys::dds_delete(self.entity) };
    }
}

impl Participant {
    pub fn new(d: libddsc_sys::dds_domainid_t) -> Participant {
        let e =
            unsafe { libddsc_sys::dds_create_participant(d, std::ptr::null(), std::ptr::null()) };
        Participant { entity: e }
    }
}
