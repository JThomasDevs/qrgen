use tuv::{QRMatrix, Module};
use bitvec::slice::BitSlice;
use bitvec::order::Lsb0;
use tuv::matrix::data_placement::place_data;

/// Print the data placement order for a version 1 matrix
#[test]
fn trace_data_placement_order() {
    let mut m = QRMatrix::new(1);
    m.place_function_patterns();
    
    // Count available data positions
    let s = m.size;
    let mut positions = Vec::new();
    
    // Simulate data placement and collect the order
    let mut col: isize = s as isize - 1;
    let mut row: isize = s as isize - 1;
    let mut going_up = true;
    let mut step = 0;
    
    while col >= 0 {
        let module = m.get(col as usize, row as usize);
        let is_data = matches!(module, Module::Data(_));
        
        if step < 50 || step > 200 {
            eprintln!("Step {:3}: ({:2}, {:2}) going_{} is_data={}", 
                     step, col, row, if going_up { "up  " } else { "down"}, is_data);
        } else if step == 50 {
            eprintln!("... (truncated)");
        }
        
        if is_data {
            positions.push((col as usize, row as usize));
        }
        
        // Move to next position
        if going_up {
            if row == 0 {
                col -= 1;
                if col < 0 { break; }
                row = 1; // Skip (col, 0) to avoid duplicate
                going_up = false;
            } else {
                row -= 1;
            }
        } else {
            if row >= (s as isize) - 1 {
                col -= 1;
                if col < 0 { break; }
                row = (s as isize) - 2; // Skip (col, s-1) to avoid duplicate
                going_up = true;
            } else {
                row += 1;
            }
        }
        
        step += 1;
        if step > 300 { break; }
    }
    
    eprintln!("\nTotal data positions available: {}", positions.len());
    eprintln!("First 20 positions: {:?}", &positions[..20]);
    eprintln!("Last 20 positions: {:?}", &positions[positions.len()-20..]);
    
    // For V1-M, we should have capacity for 16 data bytes = 128 bits
    // But we expect ~246 data modules total after function patterns
    // The spiral should place 208 bits (26 bytes with ECC)
    eprintln!("\nExpected for V1-M: 208 data bits = 208 positions");
    eprintln!("Actual positions found: {}", positions.len());
}