#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use ic_cdk::caller;


type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Gym {
    id: u64,
    gym_name : String,
    members : Vec<String>,
    owner : String,
    gym_location : String,
    gym_services : Vec<GymService>,
    gym_banner : String,
    created_at: u64,
    updated_at: Option<u64>,
}


#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct GymService {
    service_name : String,
    service_description : String,
    created_at: u64,
    updated_at: Option<u64>,
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for Gym {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for GymService {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for Gym {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for GymService {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static GYM_STORAGE: RefCell<StableBTreeMap<u64, Gym, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));


    static GYM_SERVICE_STORAGE: RefCell<StableBTreeMap<u64, GymService, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct GymPayload {
    gym_name : String,
    gym_location : String,
    gym_banner : String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct GymServicePayload {
    service_name : String,
    service_description : String,
}




#[ic_cdk::update]
fn create_gym(payload: GymPayload) -> Option<Gym> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let gym = Gym {
        id,
        owner: caller().to_string(), 
        gym_name : payload.gym_name,
        members : Vec::new(),
        gym_location : payload.gym_location,
        gym_services : Vec::new(),
        gym_banner : payload.gym_banner,
        created_at: time(),
        updated_at: None,
    };
    do_insert(&gym);
    Some(gym)
}




// #[ic_cdk::update]
// fn add_gym_service(id: u64, payload: GymServicePayload) -> Result<Gym, Error> {

//     // Retrieve the gym from the storage
//     let gym_result = GYM_STORAGE.with(|service| service.borrow().get(&id));

//     let gym_service = GymService {
//         service_name: payload.service_name,
//         service_description: payload.service_description,
//         created_at: time(),
//         updated_at: None,
//     };

//     if let Some(mut gym) = gym_result {
//         let mut gym_services = gym.gym_services.clone();
//         gym_services.push(gym_service);

//         gym.gym_services = gym_services;

//         // Update the gym in the storage
//         GYM_STORAGE.with(|service| {
//             service.borrow_mut().insert(id, gym.clone());
//         });

//         Ok(gym)
//     } else {
//         Err(Error::NotFound {
//             msg: format!("Couldn't update a gym with id={}. Gym not found", id),
//         })
//     }
// }


#[ic_cdk::update]
fn add_gym_service(id: u64, payload : GymServicePayload) -> Result<Gym, Error> {

    let gym_service = GymService {
        service_name : payload.service_name,
        service_description : payload.service_description,
        created_at: time(),
        updated_at: None,
    };

match GYM_STORAGE.with(|service| service.borrow().get(&id)) {
    Some(mut gym) => {
        
        // let mut gym_services: Vec<GymService> = gym.gym_services;
        
        gym.gym_services.push(gym_service);

        do_insert(&gym);
        // Return the modified event on success
        // Ok(gym)
        Ok(gym.clone())
    }

    // If the event is not found, return a NotFound error
    None => Err(Error::NotFound {
        msg: format!("Couldn't update an event with id={}. Event not found", id),
    }),
}
}

#[ic_cdk::query]
fn get_gym(id: u64) -> Result<Gym, Error> {
    match _get_gym(&id) {
        Some(gym) => Ok(gym),
        None => Err(Error::NotFound {
            msg: format!("gym with id={} not found", id),
        }),
    }
}








#[ic_cdk::update]
fn update_gym(id: u64, payload: GymPayload) -> Result<Gym, Error> {
    match GYM_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut gym) => {
            gym.gym_name = payload.gym_name;
            gym.gym_location = payload.gym_location;
            gym.gym_banner = payload.gym_banner;
            gym.updated_at = Some(time());
            do_insert(&gym);
            Ok(gym)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update  gym with id={}. gym not found",
                id
            ),
        }),
    }
}

// helper method to perform insert.
fn do_insert(gym: &Gym) {
    GYM_STORAGE.with(|service| service.borrow_mut().insert(gym.id, gym.clone()));
}

#[ic_cdk::update]
fn delete_gym(id: u64) -> Result<Gym, Error> {
    match GYM_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(gym) => Ok(gym),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete gym with id={}. gym not found.",
                id
            ),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// a helper method to get a message by id. used in get_message/update_message
fn _get_gym(id: &u64) -> Option<Gym> {
    GYM_STORAGE.with(|service| service.borrow().get(id))
}

// need this to generate candid
ic_cdk::export_candid!();