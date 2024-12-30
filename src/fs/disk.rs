use array_macro::array;

use super::MAX_DISKS;

pub struct Disk {}

impl Disk {
	pub const fn new() -> Self {
		Self {}
	}
}

pub struct Disks([Disk; MAX_DISKS]);

impl Disks {
	pub const fn new( ) -> Self {
		Self(array![_ => Disk::new(); MAX_DISKS])
	}
}

pub static mut DISKS = Disks::new();
