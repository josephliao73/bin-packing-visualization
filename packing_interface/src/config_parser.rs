use std::collections::{HashSet};
use rand::Rng;
use crate::types::{JsonInput, Rectangle, ParseOutput};

pub fn create_input(input: &ParseOutput) -> JsonInput {
    let mut rect_set: HashSet<(i32, i32)> = input.rects.iter().map(|r| (r.width, r.height)).collect();
    let mut final_rect_list = input.rects.clone();
    let mut rng = rand::rng();
    
    if input.autofill {
        let current_n: i32 = final_rect_list.iter().map(|r| r.quantity).sum();
        
        let mut k_delta = if input.types != -1 {
            input.types - input.input_types
        } else {
            0
        };
        
        let mut n_delta = if input.quantity != -1 {
            input.quantity - current_n
        } else {
            0
        };
        
        if input.types != -1 && k_delta > 0 {
            
            while k_delta > 0 {
                let new_x = rng.random_range(1..=input.width);
                let new_y = rng.random_range(input.min_height..=input.max_height);
                
                if rect_set.contains(&(new_x, new_y)) {
                    continue;
                }
                
                let new_rect = Rectangle { width: new_x, height: new_y, quantity: 1 };
                
                k_delta -= 1;
                n_delta -= 1;
                final_rect_list.push(new_rect);
                rect_set.insert((new_x, new_y));
            }

        }
        while n_delta > 0 {
            let rand_idx = rng.random_range(0..final_rect_list.len());
            let add = rng.random_range(1..=n_delta.max(1));
            final_rect_list[rand_idx].quantity += add; 
            n_delta -= add;
        }
    }
    
    let total_rectangles: i32 = final_rect_list.iter().map(|r| r.quantity).sum();
    
    JsonInput {
        width_of_bin: input.width,
        number_of_rectangles: total_rectangles as usize,
        number_of_types_of_rectangles: rect_set.len(),
        autofill_option: input.autofill,
        rectangle_list: final_rect_list
    }
}
