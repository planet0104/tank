use std::env;
use std::fs::File;
use std::io::{ Read,Write };

fn main() {
    let mut args = env::args();
    let _program = args.next().unwrap();
    let target_name = args.next().unwrap();

    let mut target = File::open(target_name.clone()).unwrap();
    let mut target_contents = String::new();
    target.read_to_string(&mut target_contents).unwrap();

    let mut delete = File::open(args.next().unwrap()).unwrap();
    let mut delete_contents = String::new();
    delete.read_to_string(&mut delete_contents).unwrap();
    
    for line in delete_contents.lines(){
        if line.trim().len() > 0{
            //println!("删除:{}", line);
            target_contents = target_contents.replace(line, "");
        }
    }

    let mut target = File::create(target_name).unwrap();
    target.write_all(target_contents.as_bytes()).unwrap();
}
