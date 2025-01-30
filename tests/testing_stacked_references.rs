#[cfg(test)]
mod test {
    use std::cell::{Cell, UnsafeCell};
    #[test]
    fn test_ref_stack() {
        let mut data = 10;
        let ref1 = &mut data;
        let ref2 = &mut *ref1; //ref2 is live; ref1 in stack

        *ref2 +=1;
        *ref1 +=1;

        //this does NOT work (order swapped)
        //*ref1 +=1; <- called ref1 when it's not "live" -> error
        //*ref2 +=1; 

        assert_eq!(data, 12);

        //using raw pointers we can get around this problem
        //unsafe {
        //    let mut data = 10;
        //    let ref1 = &mut data;
        //    let ref2 = ref1 as *mut _;

        //    //order swapped but still works!
        //    //miri will catch this as an error though
        //    *ref1 += 1;
        //    *ref2 += 2;

        //    println!("this works: {}", data);
        //}

        //unsafe {
        //    //stacking mutable references/raw pointers &mut -> *mut -> &mut -> *mut...
        //    let mut data = 10;
        //    let ref1 = &mut data;
        //    let ref2 = ref1 as *mut _;
        //    let ref3 = &mut *ref2;
        //    let ref4 = ref3 as *mut _;

        //    //access ref2 first
        //    //this will work, but will pop ref3 and ref4 out of stack
        //    *ref2 += 2;

        //    //if we try to access the popped refs now
        //    //miri will throw an error
        //    *ref4 += 4;
        //    *ref3 += 3;
        //    *ref2 += 2;
        //    *ref1 += 1;

        //    assert_eq!(data, 22);
        //}

        unsafe {
            let mut data = 10;
            let ref1 = &mut data;
            let ref2 = ref1 as *mut _;
            let ref3 = &mut *ref2;
            let ref4 = ref3 as *mut _;


            //this will work though
            *ref4 += 4;
            *ref3 += 3;
            *ref2 += 2;
            *ref1 += 1;

            assert_eq!(data, 20);
        }
    }

    #[test]
    fn test_arrays() {
        unsafe {
            let mut data = [0; 10];
            //only borrows the first element
            //Rust allows borrows to be broken up
            //but does not track array indeces
            //so it does not know the borrows are disjoint
            //let ref1_at_0 = &mut data[0];
            //let ref2_at_1 = &mut data[1]; <- does not work

            //we can use split_at_mut(position) for non-overlapping sub-slices
            let slice1 = &mut data[..];
            let (slice2_at_0, slice3_at_1) = slice1.split_at_mut(1);

            let ref4_at_0 = &mut slice2_at_0[0]; //data[0]
            let ref5_at_1 = &mut slice3_at_1[0]; //remember this is a slice -> data[1]
            let ptr6_at_0 = ref4_at_0 as *mut i32;
            let ptr7_at_1 = ref5_at_1 as *mut i32;

            *ptr7_at_1 += 4;
            *ptr6_at_0 += 3;
            *ref5_at_1 += 2;
            *ref4_at_0 += 1;

            println!("{:?}", &data[..]);


        }
    }

    #[test]
    fn test_slices() {
        unsafe {
            let mut data = [0; 10];
            let slice1_all = &mut data[..];
            let ptr2_all = slice1_all.as_mut_ptr();

            let ptr3_at_0 = ptr2_all;
            let ptr4_at_1 = ptr2_all.add(1);
            let ref5_at_0 = &mut *ptr3_at_0;
            let ref6_at_1 = &mut *ptr4_at_1;

            //CURRENT REF "STACK" (which is really a tree)
            // top [..] bottom
            //        [ptr2, slice1]            <- all (idx 0)
            //          |
            //          ------------------------
            //         |                       |
            // [ref5, ptr3, ptr2, slice1]  [ref6, ptr4]      <- at idx 0 / 1

            *ref6_at_1 += 4;
            *ref5_at_0 += 4;
            *ptr4_at_1 += 4;
            *ptr3_at_0 += 4;

            for idx in 0..10 {
                *ptr2_all.add(idx) += idx;
            }

            for (idx, elem_ref) in slice1_all.iter_mut().enumerate() {
                *elem_ref += idx;
            }

            println!("{:?}", &data[..]);

        }
    }

    fn opaque_read(val: &i32) {
        println!("{}", val);
    }

    #[test]
    fn test_shared_reference() {
        let mut data = 10;
        let mref1 = &mut data;
        let sref2 = &mref1;
        let sref3 = sref2;
        let sref4 = &*sref2;

        //Random hash of shared reference reads
        opaque_read(sref3);
        opaque_read(sref2);
        opaque_read(sref4);
        opaque_read(sref2);
        opaque_read(sref3);

        *mref1 += 1;
        opaque_read(&data);

        unsafe {
            let mut data = 10;
            let mref1 = &mut data;
            let ptr2 = mref1 as *mut i32;
            let sref3 = &mref1;
            // can only cast a shared reference to a *const
            // This is blasphemy but it works
            let ptr4 = *sref3 as *const i32 as *mut i32;

            // Miri will catch that ptr4 is only supposed to have ReadOnly permission
            // Once shared reference ins on the borrow stack,
            // everything that gets pushed on top of it
            // only has read permissions
            // *ptr4 += 4; <- does not work
            opaque_read(&*ptr4); // ReadOnly <- this will pass
            opaque_read(sref3);
            *ptr2 += 2;
            *mref1 += 1;
            opaque_read(&data);
        }

        //unsafe {
        //    let mut data = 10;
        //    let mref1 = &mut data;
        //    let ptr2 = mref1 as *mut i32;
        //    let sref3 = &*mref1; 

        //    *ptr2 += 2;
        //    opaque_read(sref3); // <- popped from stack; error
        //                            // Miri also states it was trying to retag for SharedReadOnly Permission
        //                            // Remember: once ANY shared reference is in the stack,
        //                            // all reference that comes on top becomes read only
        //    *mref1 += 1;

        //    opaque_read(&data);
        //}
    }

    #[test]
    fn test_cell() {
        //Cell allows values to be mutable behind a shared reference
        unsafe {
            let mut data = Cell::new(10);
            let mref1 = &mut data;
            let ptr2 = mref1 as *mut Cell<i32>;
            let sref3 = &*mref1;

            //Works even if this is not in order
            (*ptr2).set((*ptr2).get() + 2);
            sref3.set(sref3.get() + 3);
            mref1.set(mref1.get() + 1);

            opaque_read(&data.get());
        }

        //Let's try using UnsafeCell
        unsafe {
            //let mut data = UnsafeCell::new(10);
            //let mref1 = data.get_mut();           <- we're making this a regular &mut by doing this
            //let ptr2 = mref1 as *mut i32;
            //let sref3 = &*mref1;

            //*ptr2 += 2;
            //opaque_read(sref3);
            //*mref1 += 1;
            //println!("{}", *data.get());

            let mut data = UnsafeCell::new(10);
            let mref1 = &mut data;          // Mutable ref to the outside
            let ptr2 = mref1.get();                     // Raw pointer to the inside
            let sref3 = &*mref1;                // shared reference to the outside

            // as long as
            // a) REFERENCES are to the OUSIDE of cells
            // b) RAW POINTERS are to the INSIDE of cells
            // the order doesn't really matter, as can be seen below
            *ptr2 += 2;
            opaque_read(&*sref3.get());
            *sref3.get() += 3;
            *mref1.get() += 1;
            println!("{}", *data.get());
        }
    }

    #[test]
    fn test_box() {
        unsafe {
            let mut data = Box::new(10);
            let ptr1 = (&mut *data) as *mut i32;

            *ptr1 += 1;
            *data += 10;

            println!("{}", data);

        }
    }
}
