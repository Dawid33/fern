fn print_dict(name: &str, dict_rules: &HashMap<Vec<Token>, BTreeSet<Token>>, token_raw: &HashMap<Token, String>) {
    let mut f = File::create(name).unwrap();
    for (key, val) in dict_rules {
        let mut builder = String::new();
        builder.push_str("(");
        if !key.is_empty() {
            builder.push_str(format!("\'{}\'", token_raw.get(&key.get(0).unwrap()).unwrap()).as_str());
            if key.len() > 1 {
                for k in &key[1..key.len()] {
                    builder.push_str(", ");
                    builder.push_str(format!("\'{}\'", token_raw.get(&k).unwrap()).as_str());
                }
            } else {
                builder.push_str(",");
            }
            builder.push_str(") = [");

            let mut sorted = Vec::new();
            for x in val.iter() {
                sorted.push(token_raw.get(x).unwrap());
            }
            sorted.sort();

            let mut val_iter = sorted.iter();
            if val_iter.len() > 0 {
                builder.push_str(format!("\'{}\'", val_iter.next().unwrap()).as_str());
            }
            while let Some(t) = val_iter.next() {
                builder.push_str(", ");
                builder.push_str(format!("\'{}\'", t).as_str());
            }
        }
        builder.push_str("]\n");
        f.write(builder.as_bytes());
    }
}

// let mut f = File::create("rhs_dict.txt").unwrap();
// for (key, val) in &rhs_dict {
//     let mut builder = String::new();
//     builder.push_str(format!("{} = [", token_raw.get(&key).unwrap()).as_str());
//     let mut val = val.clone();
//     val.sort();
//     if !val.is_empty() {
//         let mut val_iter = val.get(0).unwrap().iter();
//         builder.push_str("[");
//         if val_iter.len() > 0 {
//             builder.push_str(format!("\'{}\'", token_raw.get(val_iter.next().unwrap()).unwrap()).as_str());
//         }
//         while let Some(t) = val_iter.next() {
//             builder.push_str(", ");
//             builder.push_str(format!("\'{}\'", token_raw.get(t).unwrap()).as_str());
//         }
//         builder.push_str("]");
//         if val.len() > 1 {
//             for k in &val[1..val.len()] {
//                 builder.push_str(", [");
//                 let mut val_iter = k.iter();
//                 if val_iter.len() > 0 {
//                     builder.push_str(format!("\'{}\'", token_raw.get(val_iter.next().unwrap()).unwrap()).as_str());
//                 }
//                 while let Some(t) = val_iter.next() {
//                     builder.push_str(", ");
//                     builder.push_str(format!("\'{}\'", token_raw.get(t).unwrap()).as_str());
//                 }
//                 builder.push_str("]");
//             }
//         }
//     }
//     builder.push_str("]\n");
//     f.write(builder.as_bytes());
// }


// let mut f = File::create("new_dict_rules.txt").unwrap();
// for (key, val) in &new_dict_rules {
//     let mut builder = String::new();
//     builder.push_str("[");
//     if !key.is_empty() {
//         let mut val_iter = key.get(0).unwrap().iter();
//         if val_iter.len() > 0 {
//             builder.push_str(format!("\'{}\'", token_raw.get(val_iter.next().unwrap()).unwrap()).as_str());
//         }
//         while let Some(t) = val_iter.next() {
//             builder.push_str(", ");
//             builder.push_str(format!("\'{}\'", token_raw.get(t).unwrap()).as_str());
//         }
//         if key.len() > 1 {
//             for k in &key[1..key.len()] {
//                 builder.push_str(", [");
//                 let mut val_iter = key.get(0).unwrap().iter();
//                 if val_iter.len() > 0 {
//                     builder.push_str(format!("\'{}\'", token_raw.get(val_iter.next().unwrap()).unwrap()).as_str());
//                 }
//                 while let Some(t) = val_iter.next() {
//                     builder.push_str(", ");
//                     builder.push_str(format!("\'{}\'", token_raw.get(t).unwrap()).as_str());
//                 }
//             }
//         }
//         builder.push_str("] = [");
//
//         let mut sorted = Vec::new();
//         for x in val.iter() {
//             sorted.push(token_raw.get(x).unwrap());
//         }
//         sorted.sort();
//
//         let mut val_iter = sorted.iter();
//         if val_iter.len() > 0 {
//             builder.push_str(format!("\'{}\'", val_iter.next().unwrap()).as_str());
//         }
//         while let Some(t) = val_iter.next() {
//             builder.push_str(", ");
//             builder.push_str(format!("\'{}\'", t).as_str());
//         }
//     }
//     builder.push_str("]\n");
//     f.write(builder.as_bytes());
// }

// let mut f = File::create("finalforreal.txt").unwrap();
// for (key, val) in &new_dict_rules{
//     let mut builder = String::new();
//     builder.push_str("[");
//     if !key.is_empty() {
//         builder.push_str("[");
//         into_str(&mut builder, key.get(0).unwrap());
//         builder.push_str("]");
//         if key.len() > 1 {
//             for k in &key[1..key.len()] {
//                 builder.push_str(", [");
//                 into_str(&mut builder, k);
//                 builder.push_str("]");
//             }
//         } else {
//             // builder.push_str(",");
//         }
//         builder.push_str("] = [");
//
//         into_str(&mut builder, &val.clone().into_iter().collect());
//     }
//     builder.push_str("]\n");
//     f.write(builder.as_bytes());
// }

// Print rules to file
// let mut f = File::create("rules.txt").unwrap();
// for r in &rules {
//     let mut line = String::from(format!("{} : [", token_raw.get(&r.left).unwrap()));
//     let mut iter = r.right.iter();
//     if let Some(x) = iter.next() {
//         line.push_str(format!("'{}'", token_raw.get(x).unwrap()).as_str());
//     }
//     while let Some(x) = iter.next() {
//         line.push_str(format!(", '{}'", token_raw.get(x).unwrap()).as_str());
//     }
//     line.push_str("]\n");
//     f.write(line.as_bytes());
// }

// let mut f = File::create("copy.txt").unwrap();
// for (key, val) in &copy {
//     let mut builder = String::new();
//     builder.push_str(format!("{} = [", token_raw.get(&key).unwrap()).as_str());
//
//     let mut sorted = Vec::new();
//     for x in val.iter() {
//         sorted.push(token_raw.get(x).unwrap());
//     }
//     sorted.sort();
//
//     let mut val_iter = sorted.iter();
//     if val_iter.len() > 0 {
//         builder.push_str(format!("\'{}\'", val_iter.next().unwrap()).as_str());
//     }
//     while let Some(t) = val_iter.next() {
//         builder.push_str(", ");
//         builder.push_str(format!("\'{}\'", t).as_str());
//     }
//     builder.push_str("]\n");
//     f.write(builder.as_bytes());
// }

// let mut f = File::create("V.txt").unwrap();
// for val in &v {
//     let mut builder = String::new();
//     builder.push_str("[");
//     let mut sorted = Vec::new();
//     for x in val.iter() {
//         sorted.push(token_raw.get(x).unwrap());
//     }
//     sorted.sort();
//
//     let mut val_iter = sorted.iter();
//     if val_iter.len() > 0 {
//         builder.push_str(format!("\'{}\'", val_iter.next().unwrap()).as_str());
//     }
//     while let Some(t) = val_iter.next() {
//         builder.push_str(", ");
//         builder.push_str(format!("\'{}\'", t).as_str());
//     }
//     builder.push_str("]\n");
//     f.write(builder.as_bytes());
// }
// let mut f = File::create("debug.txt").unwrap();
// for (key, val) in &new_dict_rules{
//     let mut builder = String::new();
//     builder.push_str("[");
//     if !key.is_empty() {
//         builder.push_str("[");
//         into_str(&mut builder, key.get(0).unwrap());
//         builder.push_str("]");
//         if key.len() > 1 {
//             for k in &key[1..key.len()] {
//                 builder.push_str(", [");
//                 into_str(&mut builder, k);
//                 builder.push_str("]");
//             }
//         } else {
//             // builder.push_str(",");
//         }
//         builder.push_str("] = [");
//
//         into_str(&mut builder, &val.clone().into_iter().collect());
//     }
//     builder.push_str("]\n");
//     f.write(builder.as_bytes());
// }

// let into_str = |builder: &mut String, input: &Vec<Token>| {
//     let mut output = Vec::new();
//     let mut should_sort = true;
//     for x in input {
//         output.push(format!("{}", token_raw.get(x).unwrap()));
//         if terminals.contains(x) {
//             should_sort = false;
//         }
//     }
//     if should_sort {
//         output.sort();
//     }
//     let mut iter = output.into_iter();
//     if let Some(x) = iter.next() {
//         builder.push_str(format!("\'{}\'", x).as_str());
//     }
//     while let Some(x) = iter.next() {
//         builder.push_str(format!(", \'{}\'", x).as_str());
//     }
// };

