#[macro_use] 
extern crate serde;
// use candid::types::principal;
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable}; 
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager,VirtualMemory}; 
use candid::{CandidType, Decode, Encode, Principal };
// use std::borrow::BorrowMut;
// use std::default;
// use std::collections::BTreeMap;
// use serde::de::value::Error;  
use std::{borrow::Cow, cell::RefCell}; 
use ic_cdk::{ query, update}; 
use std::collections::BTreeMap; 
use std::sync::Mutex; 

type Memory = VirtualMemory<DefaultMemoryImpl>; 
type IdCell = Cell<u64, Memory>; 
// type ItemStore = BTreeMap<Principal, Item>; 

// User Roles 
#[derive(CandidType, Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)] 
enum UserRole {
    #[default] 
    Empty, 
    Seller, 
    Buyer
}

// User struct
#[derive(CandidType, Serialize, Deserialize, Clone)] 
struct User {
    id: u64, 
    username: String, 
    email: String, 
    principal_id: Principal, 
    role: UserRole, 
}

impl Default for User {
    fn default() -> Self {
        Self {
         id: 0, 
         username: String::new(), 
         email: String::new(), 
         principal_id: Principal::anonymous(),
         role: UserRole::Empty, 
        }
    }   
}

// New user struct 
#[derive(CandidType, Serialize, Deserialize)] 
struct NewUser {
    username: String, 
    email: String, 
    role: UserRole
} 

// Items Struct 
#[derive(candid::CandidType, Serialize, Deserialize, Clone )] 
struct Item {
    id: u64, 
    name: String, 
    description: String, 
    amount: u64,
    principal_id: Principal, 
    sold: bool
} 

impl Default for Item {
   fn default() -> Self {
       Self {
        id: 0, 
        name: String::new(), 
        description: String::new(), 
        amount: 0, 
        principal_id: Principal::anonymous(), 
        sold: false
       }
   }   
}

// New Item struct 
#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)] 
struct NewItem {
    name: String, 
    description: String, 
    amount: u64
} 

// Serializing & Deserializing for storage and transmission 
impl Storable for Item {
   fn to_bytes(&self) -> Cow<[u8]> {
       Cow::Owned(Encode!(self).unwrap())
   }     

   fn from_bytes(bytes: Cow<[u8]>) -> Self {
       Decode!(bytes.as_ref(), Self).unwrap()
   }
}

impl Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Item {
    const MAX_SIZE: u32 = 1024; 
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for User {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    ); 
    
    static ITEM_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );
    
    static ITEM_STORAGE: RefCell<StableBTreeMap<u64, Item, Memory>> = 
    RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    )); 

    static USER_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))), 0)
            .expect("Cannot create a counter")
    );

    static USERS: RefCell<BTreeMap<Principal, User>> = RefCell::default(); 

    // static ITEMS: RefCell<ItemStore> = RefCell::default(); 

    static INSTRUCTIONS_CONSUMED: Mutex<u64> = Mutex::new(0); 

}

// For errors 
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    FieldEmpty { msg: String }, 
    Sold { msg: String }, 
    Unauthorized { msg: String }, 
    UserExists { msg: String }, 
    UserNotRegistered { msg: String }
}

// Function for registering users 
#[update]
fn register_user(new_user: NewUser) -> Result<User, Error> {
    let start = ic_cdk::api::instruction_counter(); // Start counting the number of instructions 

    if new_user.email.is_empty() || new_user.username.is_empty() || new_user.role == UserRole::Empty {
        return Err(Error::FieldEmpty { msg: format!("Kindly ensure all fields aren't empty") })
    } 

    let id = USER_COUNTER
    .with(|counter| {
        let current_value = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1)
    })
    .expect("cannot increment id counter");

    let result = USERS.with(|users| {
       let mut users_borrowed = users.borrow_mut(); 
       let principal_id_of_caller = ic_cdk::caller();  

       if users_borrowed.contains_key(&principal_id_of_caller) {
        return Err(Error::UserExists { msg: format!("User with this Principal ID already exists!") })
       }

       let user = User {
            id, 
            username: new_user.username, 
            email: new_user.email, 
            principal_id: principal_id_of_caller, 
            role: new_user.role 
        }; 

        users_borrowed.insert(principal_id_of_caller, user.clone()); 
        Ok(user)
    }); 

    let end = ic_cdk::api::instruction_counter(); // Stops counting the number of instructions 
    let instructions_consumed = end - start; // Finds the total instructions consumed 

    // Parses the instructions consumed in the "INSTRUCTIONS_CONSUMED" global variable for purposes of display 
    INSTRUCTIONS_CONSUMED.with(|instructions| {
        let mut instructions = instructions.lock().unwrap(); 
        *instructions = instructions_consumed
    }); 

    result 
}

// Displays the instructions consumed 
#[query]
fn display_instructions_consumed() -> u64 {
    INSTRUCTIONS_CONSUMED.with(|instructions| {
        let instructions = instructions.lock().unwrap(); 
        *instructions
    })
}

// Export Candid interface
ic_cdk::export_candid!();