use super::PageTable;

pub static mut KERNEL_PAGE_TABLE: PageTable = PageTable::empty();
