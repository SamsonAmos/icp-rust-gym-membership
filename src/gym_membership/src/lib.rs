#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use validator::Validate;
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use ic_cdk::caller;


type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;


// Struct definition for Gym
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Gym {
    id: u64,
    gym_name : String,
    members : Vec<GymRegistration>,
    owner : String,
    gym_location : String,
    gym_services : Vec<GymService>,
    gym_banner : String,
    created_at: u64,
    updated_at: Option<u64>,
}

// Struct definition for GymService
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct GymService {
    service_name : String,
    service_description : String,
    created_at: u64,
    updated_at: Option<u64>,
}


// Struct definition for GymRegistration
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct GymRegistration {
    user_name : String,
    owner : String,
    created_at: u64,
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


// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for GymRegistration {
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

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for GymRegistration {
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

    static GYM_REGISTRATION: RefCell<StableBTreeMap<u64, GymRegistration, Memory>> =
    RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
));
}


// Struct definition for GymPayload
#[derive(candid::CandidType, Serialize, Deserialize, Default, Validate)]
struct GymPayload {
    #[validate(length(min = 1))]
    gym_name : String,
    #[validate(length(min = 2))]
    gym_location : String,
    #[validate(length(min = 1))]
    gym_banner : String,
}


// Struct definition for GymServicePayload
#[derive(candid::CandidType, Serialize, Deserialize, Default, Validate)]
struct GymServicePayload {
    #[validate(length(min = 1))]
    service_name : String,
    #[validate(length(min = 10))]
    service_description : String,
}


// Struct definition for GymRegistrationPayload
#[derive(candid::CandidType, Serialize, Deserialize, Default, Validate)]
struct GymRegistrationPayload {
    #[validate(length(min = 1))]
    user_name : String,
}

// Function for creating a gym
#[ic_cdk::update]
fn create_gym(payload: GymPayload) -> Result<Gym, Error> {
    let check_payload = payload.validate();
    if check_payload.is_err(){
        return Err(Error:: PayloadInvalid{msg: check_payload.unwrap_err().to_string()})
    }
    // Increment the ID counter to generate a new unique ID for the gym
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
        
    // Create a new Gym instance using the provided payload and other default values
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
    Ok(gym)
}


// Function for registering for a gym
#[ic_cdk::update]
fn register_for_a_gym(id: u64, payload: GymRegistrationPayload) -> Result<Gym, Error> {
    let check_payload = payload.validate();
    if check_payload.is_err(){
        return Err(Error:: PayloadInvalid{msg: check_payload.unwrap_err().to_string()})
    }
    // GymRegistration object using the provided payload and caller's information
    let gym_registration = GymRegistration {
        user_name : payload.user_name,
        owner: caller().to_string(),
        created_at: time(),
    };

    //  Find the gym by ID in the storage
    match GYM_STORAGE.with(|service| service.borrow().get(&id)) {
    Some(mut gym) => {

    let is_already_member = gym.members.iter().any(|member| member.owner == gym_registration.owner);
    if is_already_member{
        return Err(Error::AlreadyMember { msg: format!("Caller={} is already a member of this gym.", caller()) })
    }
        
    // Add the gym registration to the gym's members list
    gym.members.push(gym_registration);
    
    // Update the gym in the storage
    do_insert(&gym);

    Ok(gym.clone())
    }

    // If the gym is not found, return a NotFound error
    None => Err(Error::NotFound {
        msg: format!("Couldn't update  gym with id={}. Gym not found", id),
    }),
}
}


// Function for adding services to a gym
#[ic_cdk::update]
fn add_gym_service(id: u64, payload : GymServicePayload) -> Result<Gym, Error> {
    let check_payload = payload.validate();
    if check_payload.is_err(){
        return Err(Error:: PayloadInvalid{msg: check_payload.unwrap_err().to_string()})
    }
    match GYM_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut gym) => {
        // Checks if the caller is the owner of the gym
            if gym.owner != caller().to_string(){
                return Err(Error::NotAuthorized {
                    msg: format!("You are not the owner"),
                });
            }
    else {
    // GymService object using the provided payload and caller's information
    let gym_service = GymService {
        service_name : payload.service_name,
        service_description : payload.service_description,
        created_at: time(),
        updated_at: None,
    };

    // Add the gym service to the gym's service list
        gym.gym_services.push(gym_service);
       
    // Update the gym in the storage
        do_insert(&gym);
        Ok(gym.clone())
    }}

    // If the gym is not found, return a NotFound error
    None => Err(Error::NotFound {
        msg: format!("Couldn't update an event with id={}. Event not found", id),
    }),
}
}


// Function  for getting gym details by it's ID 
#[ic_cdk::query]
fn get_gym(id: u64) -> Result<Gym, Error> {

    // Retrieve the gym details by using _get_gym helper function
    match _get_gym(&id) {
        // Return the gym details if found
        Some(gym) => Ok(gym),
        
        // If the gym is not found, return a NotFound error
        None => Err(Error::NotFound {
            msg: format!("gym with id={} not found", id),
        }),
    }
}


// Function for retrieving all gyms 
#[ic_cdk::query]
fn get_all_gyms() -> Result<Vec<Gym>, Error> {
    
    // Retrieve all gyms and their IDs from the GYM_STORAGE
    let gyms: Vec<(u64, Gym)> = GYM_STORAGE.with(|service| service.borrow().iter().collect());
    
    // Extract only the gym details (values) from the gyms tuple
    let gym_list: Vec<Gym> = gyms.into_iter().map(|(_, gym)| gym).collect();

    if !gym_list.is_empty() {
    // Return the list of gyms if it's not empty
        Ok(gym_list) 
    } else {
            // If the gym list is empty, return a NotFound error
        Err(Error::NotFound {
            msg: format!("no gym avaliable"),
        }) 
    }
}


// Function to update a gym by it's ID
#[ic_cdk::update]
fn update_gym(id: u64, payload: GymPayload) -> Result<Gym, Error> {
    match GYM_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut gym) => {
            let check_payload = payload.validate();
            if check_payload.is_err(){
                return Err(Error:: PayloadInvalid{msg: check_payload.unwrap_err().to_string()})
            }
        // Checks if the caller is the owner of the gym
            if gym.owner != caller().to_string(){
                return Err(Error::NotAuthorized {
                    msg: format!("You are not the owner"),
                });
            }
            

            else {
            // Update gym details with the provided payload
            gym.gym_name = payload.gym_name;
            gym.gym_location = payload.gym_location;
            gym.gym_banner = payload.gym_banner;
            gym.updated_at = Some(time());
            do_insert(&gym);
            Ok(gym)
            }
        }

        // If the gym is not found, return a NotFound error
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update  gym with id={}. gym not found",
                id
            ),
        }),
    }
}


// Function to delete a gym by it's ID
#[ic_cdk::update]
fn delete_gym(id: u64) -> Result<Gym, Error> {
    match GYM_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(gym) => {
            // Checks if the caller is not the owner of the gym
            if gym.owner != caller().to_string(){
                return Err(Error::NotAuthorized {
                    msg: format!("You are not the owner"),
                });
            }
            else {
                match GYM_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
                    Some(gym) => Ok(gym),
                    None => Err(Error::NotFound {
                    // If the gym is not found, return a NotFound error
                        msg: format!(
                            "couldn't delete gym with id={}. gym not found.",
                            id
                        ),
                    }),
                }
            }}
        None => Err(Error::NotFound {
        // If the gym is not found, return a NotFound error
            msg: format!(
                "couldn't update  gym with id={}. gym not found",
                id
            ),
        }),
}}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    NotAuthorized { msg: String },
    PayloadInvalid {msg: String},
    AlreadyMember {msg: String}
}

// helper method to perform insert.
fn do_insert(gym: &Gym) {
    GYM_STORAGE.with(|service| service.borrow_mut().insert(gym.id, gym.clone()));
}

// a helper method to get a message by id. used in get_message/update_message
fn _get_gym(id: &u64) -> Option<Gym> {
    GYM_STORAGE.with(|service| service.borrow().get(id))
}

// need this to generate candid
ic_cdk::export_candid!();
