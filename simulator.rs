use std::vec;
use std::io;
use std::collections::HashMap;
use std::cell::RefCell;

thread_local!(static GLOBAL_MAP1: RefCell<HashMap<String, Device>> = RefCell::new(HashMap::new()));

thread_local!(static GLOBAL_MAP2: RefCell<HashMap<Device, Vec<Device>>> = RefCell::new(HashMap::new()));

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
enum IPAddrKind {
    V4(u8, u8, u8, u8),
    V6(String),
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
enum DeviceType {
    Desktop(IPAddrKind),
    Switch(IPAddrKind),
    Router(IPAddrKind),
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct Device {
    dtype: DeviceType,
    name: String,
}

impl Device {
    fn new(info_vec: Vec<&str>) -> Device {
        let device_typ = match info_vec[1] {
            "Desktop" => DeviceType::Desktop(Device::evaluate_ipkind(info_vec[2], info_vec[3])),
            "Switch" => DeviceType::Switch(Device::evaluate_ipkind(info_vec[2], info_vec[3])),
            "Router" => DeviceType::Router(Device::evaluate_ipkind(info_vec[2], info_vec[3])),
            _ => panic!("Invalid Device Type")
        };
        return Device { dtype: device_typ, name: info_vec[0].to_owned() }
    }

    fn evaluate_ipkind(ip_kind: &str, ip_value: &str) -> IPAddrKind {
        match ip_kind {
            "V4" => {
                let ip_vec: Vec<&str> = ip_value.split(".").collect();
                IPAddrKind::V4(ip_vec[0].parse::<u8>().unwrap(),
                               ip_vec[1].parse::<u8>().unwrap(),
                               ip_vec[2].parse::<u8>().unwrap(),
                               ip_vec[3].parse::<u8>().unwrap())
            },
            "V6" => {
                IPAddrKind::V6(ip_value.to_owned())
            },
            _ => panic!("Invalid IP address type")
        }
    }
}

fn dir_conn_to(query: String) -> Vec<String> {
    let mut result = Vec::new();
    GLOBAL_MAP1.with(|glob_map1| {
        let map1: &HashMap<String, Device> = &*glob_map1.borrow();
        let device: Device = map1.get(&query).unwrap().clone();

        GLOBAL_MAP2.with(|glob_map2| {
            let map2: &HashMap<Device, Vec<Device>> = &*glob_map2.borrow();
            let mut connected_devices: Vec<Device> = map2.get(&device).unwrap().clone();
            connected_devices.iter_mut().for_each(|x| {
                result.push(x.name.to_owned());
            });
        });
    });
    result
}

fn find_ip_kind(query: IPAddrKind) -> Vec<String> {
    let mut result = Vec::new();

    GLOBAL_MAP1.with(|glob_map1| {
        let map1: &HashMap<String, Device> = &*glob_map1.borrow();
        match query {
            IPAddrKind::V4(_, _, _, _) => {
                let v4_devices =  map1.values().filter(|x|{
                    match x.dtype {
                        DeviceType::Desktop(IPAddrKind::V4(_, _, _, _)) => {true},
                        DeviceType::Switch(IPAddrKind::V4(_, _, _, _)) => {true},
                        DeviceType::Router(IPAddrKind::V4(_, _, _, _)) => {true},
                        _ => {false}
                    }
                });
                for device in v4_devices {
                    result.push(device.name.clone());
                }
            },
            IPAddrKind::V6(_) => {
                let v6_devices = map1.values().filter(|x| {
                    match x.dtype {
                        DeviceType::Desktop(IPAddrKind::V6(_)) => {true},
                        DeviceType::Switch(IPAddrKind::V6(_)) => {true},
                        DeviceType::Router(IPAddrKind::V6(_)) => {true},
                        _ => {false}
                    }
                });
                for device in v6_devices {
                    result.push(device.name.clone());
                }
            }
        }
    });
    result
}

fn can_talk(src: String, dst: String) -> Vec<String> {
    let mut result = Vec::new();
    let mut visited_vec: Vec<Device> = Vec::new();

    GLOBAL_MAP1.with(|glob_map| {
        let map1: &HashMap<String, Device> = &*glob_map.borrow();
        let s: Device = map1.get(&src).unwrap().clone();
        let d: Device = map1.get(&dst).unwrap().clone();

        let dfs_result = visit_node(s, d, &mut visited_vec);
        if dfs_result == String::from("") { }
        else {
            let route = src+" "+&dfs_result;
            let mut route_vec: Vec<&str> = route.split(" ").collect();
            route_vec.iter_mut().for_each(|x| {
                result.push(x.to_owned());
            });
        }
        
    });
    result
}

fn rm_device(device: String) -> bool {

    GLOBAL_MAP1.with(|glob_map1| {
        let map1: &mut HashMap<String, Device> = &mut *glob_map1.borrow_mut();
        GLOBAL_MAP2.with(|glob_map2| {
            let map2: &mut HashMap<Device, Vec<Device>> = &mut *glob_map2.borrow_mut();
            
            let rem_dev = map1.get(&device).unwrap().clone();
            map2.remove(&rem_dev);
            for (key, values) in &mut *map2 {
                values.retain(|x| x != &rem_dev);
            }
            map1.remove(&device);
            // println!("{:?}", map1);
            // println!("{:?}", map2);
        });
    });
    true
}

fn main() {
    //Temp map for unidirectional connections
    let mut temp_map: HashMap<Device, Vec<Device>> = HashMap::new();

    // inputs will be taken from standard i/o. A file has attached to sample.
    println!("Number of Devices:");
    let mut device_cnt = String::new();
    io::stdin().read_line(&mut device_cnt).expect("failed to readline"); 

    for index in 0..(device_cnt.trim().parse::<i32>().unwrap()) {
        println!("Device Info:");
        let mut device_info = String::new();
        io::stdin().read_line(&mut device_info).expect("failed to readline");
        device_info = device_info.trim().to_owned();
        //device_info = "Switch1 Switch V6 2001:0db8:85a3:0000:0000:8a2e:0370:7334".to_owned();

        let device_info_vec: Vec<&str> = device_info.split(" ").collect();
        let device = Device::new(device_info_vec);
        println!("Created {:?}", device);

        //Insert device into global map<String, Device>
        GLOBAL_MAP1.with(|glob_map| {
            let map1: &mut HashMap<String, Device> = &mut *glob_map.borrow_mut();
            map1.insert(device.clone().name, device.clone());
        });

        while true {
            println!("Number of Connections:");
            let mut connectn_ct = String::new();
            io::stdin().read_line(&mut connectn_ct).expect("failed to readline");
            let conn_count = connectn_ct.trim().parse::<usize>().unwrap();

            let mut size = 0;
            GLOBAL_MAP1.with(|glob_map| {
                let map1: &HashMap<String, Device> = &*glob_map.borrow();
                size = map1.len();
            });

            if conn_count >= size {
                println!("Insufficient Devices, Try again!!");
            } else if conn_count == 0 {
                break;
            } else {
                while true {
                    println!("Connections are:");
                    let mut connectn_info = String::new();
                    io::stdin().read_line(&mut connectn_info).expect("failed to readline");
                    connectn_info = connectn_info.trim().to_owned();
                    //connectn_info = "Desk1 Switch1".to_owned();
                    
                    let mut connectn_info_vec: Vec<&str> = connectn_info.split(" ").collect();
                    if conn_count != connectn_info_vec.len() {
                        println!("Incorrect count of connections provided, Try again!!");
                        continue;
                    } 

                    let mut info_result = true;
                    
                    connectn_info_vec.iter_mut().for_each(|x| {
                        GLOBAL_MAP1.with(|glob_map| {
                            let map1: &HashMap<String, Device> = &*glob_map.borrow();
                            if map1.contains_key(x.clone()) { }
                            else { //Device not present
                                info_result = false;
                            }
                        });
                    });

                    if info_result == false {
                        println!("Incorrect connections (one or more devices not found), Try again!!");
                    } else {
                        //ENTIRE LOGIC TO ADD(perform) CONNECTION GOES HERE
                        perform_connections(device, connectn_info_vec, &mut temp_map);
                        println!("This device connections added sucessfully !!!");
                        break;
                    }
                }
                break;
            }
        }

    }
    
    //Refactor unidirectional connections to bidirectional and store in GLOBAL_MAP2
    connect_bidirectional(&temp_map);
    loop{
        //Now call all the functions one by one
        println!("Choice operation:\n
            1) Who are directly connected to <input>?\n
            2) Who are using <input> IP Kind?\n
            3) How does <input1> and <input2> can talk?\n
            4) Remove the <input>?\n");
        println!("Choice:");
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("failed to readline");
    
        match choice.trim() {
            "1" => {
                let input = read_input();
                let ans = dir_conn_to(input);
                println!("Answer: {:?}", ans);
            },
            "2" => {
                let input: &str = &read_input();
                let ans = match input {
                    "V4" => { find_ip_kind(IPAddrKind::V4(0, 0, 0, 0)) },
                    "V6" => { find_ip_kind(IPAddrKind::V6(String::from(""))) },
                    _ => { println!("Invalid IpAddrKind");
                           Vec::<String>::new() }
                };
                println!("Answer: {:?}", ans);
            },
            "3" => {
                let input = read_input();
                let input_vectr: Vec<&str> = input.split(" ").collect();
                let mut ans: Vec<String> = can_talk(String::from(input_vectr[0]), String::from(input_vectr[1]));
                if ans.is_empty() {
                    println!("Answer: No connection.");
                } else {
                    print!("\nAnswer: ");
                    ans.iter_mut().for_each(|x| print!("{:?} ->", x));
                    println!("");
                }
            },
            "4" => {
                let input = read_input();
                let ans = rm_device(input);
                println!("Answer: {:?}", ans);
            },
            _ => {
                println!("Invaid operation");
            }
        }
    }



    assert_eq!(
        vec![
            String::from("Switch1"),
            String::from("Desk2"),
            String::from("Switch2")
        ],
        dir_conn_to(String::from("Router1"))
    );

    assert_eq!(
        vec![String::from("Switch2"), String::from("Switch1")],
        find_ip_kind(IPAddrKind::V6(String::from("")))
    );

    assert_eq!(
        vec![
            String::from("Desk1"),
            String::from("Switch1"),
            String::from("Router1"),
            String::from("Switch2"),
            String::from("Desk3")
        ],
        can_talk(String::from("Desk1"), String::from("Desk3"))
    );

    assert!(rm_device(String::from("Switch2")));

    let exp_vec: Vec<String> = Vec::new();
    assert_eq!(
        exp_vec,
        can_talk(String::from("Desk1"), String::from("Desk3"))
    );
}

fn perform_connections(device: Device, mut connectn_info_vec: Vec<&str>, temp_map: &mut HashMap<Device, Vec<Device>>) {
    let mut connections: Vec<Device> = Vec::new();

    connectn_info_vec.iter_mut().for_each(|x| {
        GLOBAL_MAP1.with(|glob_map| {
            let map1: &HashMap<String, Device> = &*glob_map.borrow();
            connections.push(map1.get(x.clone()).unwrap().clone());
        });
    });
    temp_map.insert(device, connections);
}

fn connect_bidirectional(temp_map: &HashMap<Device, Vec<Device>>) {
    
    GLOBAL_MAP1.with(|glob_map1| {
        let map1: &HashMap<String, Device> = &*glob_map1.borrow(); //Just use for searching/finding device by name/device count etc

        GLOBAL_MAP2.with(|glob_map2| {
            let map2: &mut HashMap<Device, Vec<Device>> = &mut *glob_map2.borrow_mut();
            
            for (key, val) in map1 {
                map2.insert(val.clone(), vec![]);
            }

            for (key, values) in temp_map {
                map2.insert(key.clone(), values.clone());   
            }

            for (key, values) in temp_map {
                for val in values {
                    let mut valVect = &mut map2.get(val).unwrap().to_owned();
                    valVect.push(key.to_owned());
                    map2.insert(val.to_owned(), valVect.to_vec());
                }
            }
        });
    });
}

fn read_input() -> String {
    println!("Input:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("failed to readline");
    input.trim().to_owned()
}

//Recursive DFS (Depth First Search) graph algorithm used here to find the path between two nodes.
fn visit_node(root: Device, dest: Device, visited_vec: &mut Vec<Device>) -> String {
    visited_vec.push(root.clone());

    let mut con_vec: Vec<Device> = Vec::new();
    GLOBAL_MAP2.with(|glob_map| {
        let map2: &HashMap<Device, Vec<Device>> = &*glob_map.borrow();
        con_vec = map2.get(&root).unwrap().to_owned();
    });

    for node in con_vec {
        if node == dest { return node.name; } 
        else {
            if !visited_vec.contains(&node) {
                let mut result = visit_node(node.to_owned(), dest.to_owned(), visited_vec);

                if result == "".to_owned() {
                    continue;
                } else {
                    result = node.name +" " +&result;
                    return result;
                }
            }
        }
    }
    return "".to_owned();
}