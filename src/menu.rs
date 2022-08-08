use crate::prelude::*;

pub fn menu_box<T: ToString>(
    batch: &mut DrawBatch,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    title: T,
    footer: Option<T>,
) {
    batch.draw_box(
        Rect::with_size(x, y - 2, width, height),
        ColorPair::new(WHITE, BLACK),
    );
    batch.print_color(
        Point::new(x + 3, y - 2),
        &title.to_string(),
        ColorPair::new(MAGENTA, BLACK),
    );
    if let Some(footer) = footer {
        batch.print_color(
            Point::new(x + 3, y + height - 1),
            footer.to_string(),
            ColorPair::new(YELLOW, BLACK),
        );
    }
}

pub fn menu_option<T: ToString>(
    batch: &mut DrawBatch,
    x: i32,
    y: i32,
    hotkey: FontCharType,
    text: T,
) {
    batch.set(
        Point::new(x, y),
        ColorPair::new(WHITE, BLACK),
        to_cp437('('),
    );
    batch.set(Point::new(x + 1, y), ColorPair::new(YELLOW, BLACK), hotkey);
    batch.set(
        Point::new(x + 2, y),
        ColorPair::new(WHITE, BLACK),
        to_cp437(')'),
    );
    batch.print_color(
        Point::new(x + 5, y),
        &text.to_string(),
        ColorPair::new(YELLOW, BLACK),
    );
}

pub fn render_menu<T: ToString>(
    batch: &mut DrawBatch,
    x: i32,
    y: i32,
    width: i32,
    title: T,
    footer: Option<T>,
    items: &[(char, T)],
) {
    let mut y = y;
    menu_box(batch, x, y, width, (items.len() + 3) as i32, title, footer);
    for item in items.iter() {
        menu_option(batch, 17, y, to_cp437(item.0), item.1.to_string());
        y += 1;
    }
}

pub enum ItemMenuResult {
    NoResponse,
    Cancel,
    Selected,
}

pub fn item_result_menu<S: ToString>(
    batch: &mut DrawBatch,
    title: S,
    count: usize,
    items: &[(Entity, String)],
    key: Option<VirtualKeyCode>,
) -> (ItemMenuResult, Option<Entity>) {
    // determine required width
    // 2 for each border+margin, 4 for key+space
    // 20 was the original width
    let width = usize::max(
        items.iter().map(|i| i.1.len()).max().unwrap_or(20) + 2 + 2 + 4,
        20,
    ) as i32;
    let mut y = (25 - (count / 2)) as i32;
    menu_box(batch, 15, y, width, (count + 3) as i32, title, None);
    batch.print_color(
        Point::new(18, y + count as i32 + 1),
        "ESCAPE to cancel",
        ColorPair::new(YELLOW, BLACK),
    );

    let mut item_list: Vec<Entity> = Vec::new();
    let mut j = 0;
    for item in items {
        menu_option(batch, 17, y, 97 + j as FontCharType, &item.1);
        item_list.push(item.0);
        y += 1;
        j += 1;
    }

    batch.submit(12000).expect("Batch error");

    match key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(item_list[selection as usize]),
                    );
                } else {
                    (ItemMenuResult::NoResponse, None)
                }
            }
        },
    }
}

pub enum VendorMenuResult {
    NoResponse,
    Cancel,
    Selected,
    BuySellToggle,
    NextPage,
    PreviousPage,
}

pub fn vendor_result_menu<S: ToString>(
    batch: &mut DrawBatch,
    title: S,
    page: i32,
    items: &[(String, f32)],
    key: Option<VirtualKeyCode>,
) -> (VendorMenuResult, Option<usize>) {
    // determine required width
    // 2 for each border+margin, 4 for key+space, 7 for price + space
    // 51 was the original width
    let width = usize::max(
        items.iter().map(|i| i.0.len()).max().unwrap_or(20) + 2 + 2 + 4 + 7,
        51,
    ) as i32;

    // maximum 20 items per page
    let max_per_page = 20;
    let total = items.len();
    let count = i32::min(total as i32 - (page * max_per_page), max_per_page) as usize;
    let mut y = (25 - (count / 2)) as i32;

    let first_page = page == 0;
    let last_page = items.len() < ((page + 1) * max_per_page) as usize;

    menu_box(batch, 15, y, 51, (count + 3) as i32, title, None);
    batch.print_color(
        Point::new(18, y + count as i32 + 1),
        "ESCAPE to cancel",
        ColorPair::new(YELLOW, BLACK),
    );

    let mut item_list: Vec<usize> = Vec::new();
    for j in 0..count {
        let item = &items[j + (page * max_per_page) as usize];
        menu_option(batch, 17, y, 97 + j as FontCharType, &item.0);
        batch.print(Point::new(50, y), &format!("{:.1} gp", item.1));
        item_list.push(j);
        y += 1;
    }

    // multiple pages?
    if items.len() > max_per_page as usize {
        // page switchers.
        let text = if first_page {
            // first page
            format!("(1) ==>")
        } else if last_page {
            // last page
            format!("<== ({})", page + 1)
        } else {
            format!("<== ({}) ==>", page + 1)
        };

        batch.print_right(Point::new(15 + width - 2, y), text);
    }

    batch.submit(12000).expect("Batch error");

    match key {
        None => (VendorMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (VendorMenuResult::Cancel, None),
            VirtualKeyCode::Space => (VendorMenuResult::BuySellToggle, None),
            VirtualKeyCode::Left => {
                if first_page {
                    (VendorMenuResult::NoResponse, None)
                } else {
                    (VendorMenuResult::PreviousPage, None)
                }
            }
            VirtualKeyCode::Right => {
                if last_page {
                    (VendorMenuResult::NoResponse, None)
                } else {
                    (VendorMenuResult::NextPage, None)
                }
            }
            _ => {
                let selection = letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        VendorMenuResult::Selected,
                        Some(item_list[selection as usize]),
                    );
                } else {
                    (VendorMenuResult::NoResponse, None)
                }
            }
        },
    }
}
