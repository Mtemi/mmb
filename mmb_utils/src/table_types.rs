use parking_lot::Mutex;
use paste::paste;
use std::collections::HashMap;

use crate::infrastructure::WithExpect;
// Declare methods add_or_get and get_str for structs AppendTable8 and AppendTable16 according to its item bit count
// This types keep items in static tables with fixed sizes
// AppendTable8 (size = 256)
// AppendTable16 (size = 65536)
macro_rules! impl_append_table {
    ($bits: literal) => {
        paste! {
            impl [<AppendTable $bits>] {
                /// Added leaked copy of string to table
                /// Return table index of string equal to specified value if found otherwise create leaked
                /// clone of string, add it to table and return index in table for new value
                pub fn add_or_get(&self, value: &str) -> [<u $bits>] {
                    let mut inner = self.0.lock();
                    let map = &mut inner.map;

                    if let Some(&index) = map.get(value) {
                        return index;
                    }

                    let count = map.len();
                    let index: [<u $bits>] = count.try_into().with_expect(|| {
                        format!(
                            "For AppendTable{} index {} is greater then {} (map: {:?})",
                            $bits, count, [<u $bits>]::MAX, map
                        )
                    });

                    let static_str: &'static str = Box::leak(value.to_owned().into_boxed_str());

                    let _ = map.insert(static_str, index);

                    let str_ptr = inner
                        .buffer
                        .get_mut(index as usize)
                        .expect("index should be inside because we check it in previous step");
                    *str_ptr = Some(static_str);

                    index
                }

                pub fn get_str(&self, index: [<u $bits>]) -> &'static str {
                    // SAFE: use unsafe just for access to item of append only table
                    let inner = unsafe { self.0.data_ptr().as_ref() }
                        .with_expect(|| format!("Can't get pointer to AppendTable{}Inner data", $bits));

                    // Should exist because it assigned when created table item instance
                    inner // can read without lock because buffer is append only table
                        .buffer
                        .get(index as usize)
                        .with_expect(|| format!("Index {} out of bounds of AppendTable{}", index, $bits)) // Index should be correct because we get it when created table item instance
                        .with_expect(|| {
                            format!(
                                "Unable to get string from AppendTable{}[{}] because it's None (table: {:?})",
                                $bits,
                                index,
                                self.0.lock()
                            )
                        })
                }
            }
        }
    };
}

#[derive(Debug)]
struct AppendTable8Inner {
    // supplementary map for fast checking existence same string when creating new AppendTable8 item
    map: HashMap<&'static str, u8>,
    // buffer for append only items
    buffer: [Option<&'static str>; 256],
}

pub struct AppendTable8(Mutex<AppendTable8Inner>);

/// Append only table for keeping small number (<= 256) of strings with different values
/// Expected using as static table (live whole time that program work)
impl AppendTable8 {
    pub fn new() -> Self {
        AppendTable8(Mutex::new(AppendTable8Inner {
            buffer: [None; 256],
            // creating oversize map for not resizing in future
            map: HashMap::with_capacity(512),
        }))
    }
}

impl_append_table!(8);

#[derive(Debug)]
struct AppendTable16Inner {
    // supplementary map for fast checking existence same string when creating new AppendTable16 item
    map: HashMap<&'static str, u16>,
    // buffer for append only items
    // (Use Vec because array fails with stackoverflow)
    buffer: Vec<Option<&'static str>>,
}

pub struct AppendTable16(Mutex<AppendTable16Inner>);

/// Append only table for keeping small number (<= 65536) of strings with different values
/// Expected using as static table (live whole time that program work)
impl AppendTable16 {
    pub fn new() -> Self {
        AppendTable16(Mutex::new(AppendTable16Inner {
            buffer: vec![None; 65536],
            map: HashMap::new(),
        }))
    }
}

impl_append_table!(16);
