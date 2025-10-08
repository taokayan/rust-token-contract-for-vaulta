
#[no_mangle]
extern "C" {
    pub fn prints_l(data: *const u8, len: u32);
    pub fn require_auth(name: u64);
    pub fn eosio_assert_message(test: u32, msg: *const u8, len :u32 );
    pub fn current_time()-> u64;
    pub fn get_block_num()-> u32;
    pub fn get_sender()-> u64;

    // serialization in Vaulta format:
    // - primitive:
    //   - i8,u8,i16,u16,i32,u32,i64,u64,float32,double64: little endian fixed-byte copy 
    //   - all currency amounts are using u64
    // - account names / action names / short names:
    //   - An account / action / short name is an u64 and serialized the same way as u64.
    //     When converting to string, it uses based-32 conversion using chars [.1-5a-z].
    //     max 12 chars long (12 x 5bit = 60 bit)
    //     lowest 4 bits always 0.
    // - symbol name (with or without precision):
    //   - symbol name is stored as u64 and serialized the same way as u64. 
    //     When converting to string, the first 7 bytes represent the symbol (ASCII value, always
    //     upper case), and the last byte represents the precision.
    // - usize, size of vectors/arrays:
    //   - variable size: each byte using the highest 1-bit as stop bit, the rest 7-bit for value
    //                    for example, 127 -> [0x7f]
    // - string(treated as vector of u8), vector of T:
    //   - serialize the size of string/vector as 'usize'
    //   - serialize each element
    // - variant types of [T0, T1, T2]:
    //   - serialize the type index i (as usize)
    //   - serialize the element in the i-th type.

    // read serialized action data
    pub fn read_action_data(msg: *mut u8, len: u32 )-> u32;

    // store a new key(u64)-value(binary) record inside (table name n and scope s) 
    // and return iterator to new entry
    pub fn db_store_i64(scope: u64, table_name: u64, ram_payer: u64, 
        primary_key: u64, data: *const u8, len: u32)-> i32;

    // update value by iterator
    pub fn db_update_i64(iterator:i32, ram_payer:u64, data: *const u8, len:u32);

    // remove record;
    pub fn db_remove_i64(iterator:i32);

    // get stored binary data by iterator
    // return real len stored in database, 
    // if real len > len, retrieved data is truncated and should be called again
    pub fn db_get_i64(iterator:i32, data: *mut u8, len:u32)->i32;

    // get iterator to record by primary key
    // return negative if key not found
    pub fn db_find_i64(code: u64, scope: u64, table_name: u64, primary_key: u64)->i32;

    // get lower/upper bound iterator, sorted by primary key
    // return negative if not found or reach the end
    pub fn db_lowerbound_i64(code: u64, scope: u64, table_name: u64, primary_key: u64)->i32;
    pub fn db_upperbound_i64(code: u64, scope: u64, table_name: u64, primary_key: u64)->i32;

    // get end iterator of a table, usualy returns a negative number
    pub fn db_end_i64(code: u64, scope: u64, table_name: u64)->i32;

    // get next/previous iterator and primay_key by the current iterator
    pub fn db_next_i64(iterator:i32, next_primary: *mut u64)->i32;
    pub fn db_previous_i64(iterator:i32, next_primary: *mut u64)->i32;
}

fn read_action_data_as_vec()->Vec<u8> {
    unsafe {
        let len:u32 = read_action_data(0 as *mut u8, 0);
        let mut v: Vec<u8> = Vec::new();
        v.resize(len as usize, 0);
        if (len > 0) {
            read_action_data(v.as_mut_ptr(), len);
        }
        return v;
    }
}

fn stream_append_u64(data: u64, stream: &mut Vec<u8>) {
    stream.push((data & 0xff) as u8);
    stream.push(((data>>8) & 0xff) as u8);
    stream.push(((data>>16) & 0xff) as u8);
    stream.push(((data>>24) & 0xff) as u8);
    stream.push(((data>>32) & 0xff) as u8);
    stream.push(((data>>40) & 0xff) as u8);
    stream.push(((data>>48) & 0xff) as u8);
    stream.push(((data>>56) & 0xff) as u8);
}

fn stream_read_u64(stream: &Vec<u8>, offset: usize)->(u64, usize) {
    let mut val:u64 = 0;
    val = val + stream[offset + 7] as u64;  val = (val << 8);
    val = val + stream[offset + 6] as u64;  val = (val << 8);
    val = val + stream[offset + 5] as u64;  val = (val << 8);
    val = val + stream[offset + 4] as u64;  val = (val << 8);
    val = val + stream[offset + 3] as u64;  val = (val << 8);
    val = val + stream[offset + 2] as u64;  val = (val << 8);
    val = val + stream[offset + 1] as u64;  val = (val << 8);
    val = val + stream[offset] as u64; 
    return (val, 8);
}

// standard table format for Vaulta asset row:
// table name = accounts, scope = user, primary key = symbol_code
struct asset_t {
    amount: i64,
    symbol_code: u64, // high-56bit: string, low-8bit: precision
}
fn stream_append_asset(data: &asset_t, stream: &mut Vec<u8>) {
    stream_append_u64(data.amount as u64, stream);
    stream_append_u64(data.symbol_code, stream);
}
fn stream_read_asset(stream: &Vec<u8>, offset: usize)->(asset_t, usize) {
    let (a, o1) = stream_read_u64(&stream, offset);
    let (sc, o2) = stream_read_u64(&stream, offset+o1);
    return (asset_t{amount: a as i64, symbol_code: sc}, o1+o2); 
}

// standard table format for Vaulta currency stats row:
// table name = stat, scope = primary key = (supply.symbol_code >> 8)
struct currency_stats_t {
    supply: asset_t,
    max_supply: asset_t,
    issuer: u64,
}
fn stream_append_currency_stats(data: &currency_stats_t, stream: &mut Vec<u8>) {
    stream_append_asset(&data.supply, stream);
    stream_append_asset(&data.max_supply, stream);
    stream_append_u64(data.issuer, stream);
}
fn stream_read_currency_stats(stream: &Vec<u8>, offset: usize)->(currency_stats_t, usize) {
    let (s, o1) = stream_read_asset(&stream, offset);
    let (mx, o2) = stream_read_asset(&stream, offset+o1);
    let (iss, o3) = stream_read_u64(&stream, offset+o1+o2);
    return (currency_stats_t{supply:s, max_supply:mx, issuer:iss}, o1+o2+o3); 
}

// stardard params for currency create action
struct create_param_t {
    issuer: u64,
    max_supply: asset_t,
}
fn stream_read_create_param_t(stream: &Vec<u8>)-> create_param_t {
    let (_issuer, offset1) = stream_read_u64(&stream, 0);
    let (_max_supply, offset2) = stream_read_asset(&stream, offset1);
    let ret = create_param_t {
        issuer: _issuer, 
        max_supply: _max_supply
    };
    return ret;
}
fn action_create() {
    let stream = read_action_data_as_vec();
    let param: create_param_t = stream_read_create_param_t(&stream);
    
    let debug = true; // <-- turn debug on or off here
    if (debug) {
        let mut s = String::from("create action: issuer:");
        let s2 = u64_to_name(param.issuer);
        let s3 = s + &s2;
        print(&s3);
    }

    unsafe {
        require_auth(param.issuer);
    }

    let _symbol_code:u64 = param.max_supply.symbol_code;
    let _supply = asset_t{ amount: 0, symbol_code: _symbol_code};
    let stat = currency_stats_t { 
        supply: _supply, 
        max_supply: param.max_supply, 
        issuer: param.issuer
    };

    let mut stat_binary_data: Vec<u8> = Vec::new();
    stream_append_currency_stats(&stat, &mut stat_binary_data);

    unsafe {
        let symbol_code_without_precision:u64 = (_symbol_code >> 8);
        db_store_i64(symbol_code_without_precision, // scope
            name_to_u64("stat"),                    // table name
            param.issuer,                           // ram_payer
            symbol_code_without_precision,          // primary key
            stat_binary_data.as_ptr(),              // data
            stat_binary_data.len() as u32);         // len
    }
}

fn _add_stat(_selfcode:u64, asset:&asset_t)->currency_stats_t {
    let scope:u64 = (asset.symbol_code >> 8);
    let table_name = name_to_u64("stat");
    let primary_key = scope;

    // fetch iterator by primiary key 
    let iter:i32 = unsafe { db_find_i64(_selfcode, scope, table_name, primary_key) };
    check(iter >= 0, "unable to find the currency stats object");

    // fecth binary data by iterator
    let mut binary_data: Vec<u8> = Vec::new();
    binary_data.resize(40, 0); // supply, max supply, issuer
    let len_read:i32 = unsafe { db_get_i64(iter, binary_data.as_mut_ptr(), binary_data.len() as u32) };
    check(len_read == 40, "invalid len of stat row");
    let (mut stat_obj, len2) = stream_read_currency_stats(&binary_data, 0);
    
    // update supply amount
    check(stat_obj.supply.symbol_code == asset.symbol_code, "symbol precision mismatch");
    let new_amount = stat_obj.supply.amount + asset.amount;
    check(new_amount >= stat_obj.supply.amount, "supply overflow");
    check(new_amount <= stat_obj.max_supply.amount, "supply exceed max_supply");

    // save to db
    stat_obj.supply.amount = new_amount;
    binary_data = Vec::new();
    stream_append_currency_stats(&stat_obj, &mut binary_data);
    unsafe { db_update_i64(iter, 0 /*same ram_payer*/, binary_data.as_ptr(), binary_data.len() as u32) };

    return stat_obj;
}

// helper function
fn _add_sub_balance(_selfcode:u64, owner:u64, asset:&asset_t, ram_payer:u64, _add:bool) {
    let mut binary_data: Vec<u8> = Vec::new();
    let primary_key:u64 = (asset.symbol_code >> 8);
    let scope = owner;
    let table_name = name_to_u64("accounts");

    // db_find_i64(code: u64, scope: u64, table_name: u64, primary_key: u64)->i32;
    let iter:i32 = unsafe { db_find_i64(_selfcode, scope, table_name, primary_key) };
    if (iter < 0) { // not found
        check(_add, "no balance object found");
        stream_append_asset(asset, &mut binary_data);
        unsafe { 
            db_store_i64(scope, table_name, ram_payer, primary_key, binary_data.as_ptr(), 
                        binary_data.len() as u32);
        }
    } else {
        let mut orig_binary_data: Vec<u8> = Vec::new();
        orig_binary_data.resize(16, 0); // 64-bit amount + 64 bit symbol code
        let len_read = unsafe { db_get_i64(iter, orig_binary_data.as_mut_ptr(), orig_binary_data.len() as u32) };
        check(len_read == 16, "invalid len of balance row");
        let (mut orig_asset, len) = stream_read_asset(&orig_binary_data, 0);
        check(orig_asset.symbol_code == asset.symbol_code, "symbol precision mismatch");
        let mut new_amount = orig_asset.amount;
        if (_add) {
            new_amount += asset.amount;
            check(new_amount >= orig_asset.amount, "add balance overflow");
        } else {
            check(orig_asset.amount >= asset.amount, "overdrawn balance");
            new_amount -= asset.amount;
        }
        orig_asset.amount = new_amount;
        stream_append_asset(&orig_asset, &mut binary_data);
        if (new_amount > 0) {
            if (owner == ram_payer) {
                unsafe { db_update_i64(iter, ram_payer, binary_data.as_ptr(), binary_data.len() as u32);}
            } else {
                unsafe { db_update_i64(iter, 0 /*same ram_payer*/, binary_data.as_ptr(), binary_data.len() as u32);}
            }
        } else {
            unsafe { db_remove_i64(iter); }
        }
    }
}
fn _add_balance(_selfcode:u64, owner:u64, asset:&asset_t, ram_payer:u64) {
    _add_sub_balance(_selfcode, owner, asset, ram_payer, true);
}
fn _sub_balance(_selfcode:u64, owner:u64, asset:&asset_t, ram_payer:u64) {
    _add_sub_balance(_selfcode, owner, asset, ram_payer, false);
}

// action issue (issuer, quantity, memo)
struct issue_param_t {
    to: u64,
    quantity: asset_t,
    memo: String
}
fn stream_read_issue_param_t(stream: &Vec<u8>)-> issue_param_t {
    let (_to, offset1) = stream_read_u64(&stream, 0);
    let (_quantity, offset2) = stream_read_asset(&stream, offset1);
    let ret = issue_param_t {
        to: _to, 
        quantity: _quantity,
        memo: String::from("") // <--- FIXME: decode memo
    };
    return ret;
}
fn action_issue(_selfcode:u64) {
    let stream = read_action_data_as_vec();
    let param: issue_param_t = stream_read_issue_param_t(&stream);
    check(param.quantity.amount > 0, "issue quantity must > 0");
    let stat:currency_stats_t = _add_stat(_selfcode, &param.quantity);
    unsafe { require_auth(stat.issuer); }
    _add_balance(_selfcode, param.to, &param.quantity, stat.issuer);
}

// action transfer (from, to, quantity, memo)
struct transfer_param_t {
    from: u64,
    to: u64,
    quantity: asset_t,
    memo: String
}
fn stream_read_transfer_param_t(stream: &Vec<u8>)-> transfer_param_t {
    let (_from, offset1) = stream_read_u64(&stream, 0);
    let (_to, offset2) = stream_read_u64(&stream, offset1);
    let (_quantity, offset3) = stream_read_asset(&stream, offset1+offset2);
    let ret = transfer_param_t {
        from: _from, 
        to: _to,
        quantity: _quantity,
        memo: String::from(""), // FIXME: decode memo
    };
    return ret;
}
fn action_transfer(_selfcode:u64) {
    let stream = read_action_data_as_vec();
    let param: transfer_param_t = stream_read_transfer_param_t(&stream);
    unsafe { require_auth(param.from); };
    check(param.quantity.amount > 0, "transfer quantity must > 0");
    _sub_balance(_selfcode, param.from, &param.quantity, param.from);
    _add_balance(_selfcode, param.to, &param.quantity, param.from);
}

fn check(test: bool, s: &str) {
    if (!test) {
        let sbytes = s.as_bytes();
        unsafe {
            eosio_assert_message(0, sbytes.as_ptr(), sbytes.len() as u32);
        }
    }
}
fn print(s: &str) {
    let sbytes = s.as_bytes();
    unsafe {
        prints_l(sbytes.as_ptr(), sbytes.len() as u32);
    }
}

fn name_to_u64(s: &str) -> u64 {
    let sbytes = s.as_bytes();
    let mut i:usize = 0;
    let mut val:u64 = 0;
    while (i < 12) {
        let mut c:u8 = 0;
        let mut v:u8 = 0;
        if (i < sbytes.len()) {
            c = (sbytes[i] as u8);
        }
        if (c >= 97 && c <= 122) {
            v = c - 97 + 6;
        } else if (c >= 48 && c <= 53) {
            v = c - 48;
        }
        val = (val << 5) + (v as u64);
        i = i + 1;
    }
    val = (val << 4);
    return val;
}

fn u64_to_name(name: u64) -> String {
    let mut s = String::from("");
    let mut v:u64 = name;
    while (v > 0) {
        let v0 = (v >> 59) as u8;
        if (v0 == 0) {
            s.push('.');
        } else if (v0 >= 1 && v0 <= 5) {
            s.push(('0' as u8 + v0) as char);
        } else {
            s.push(('a' as u8 + v0 - 6) as char);
        }
        v = (v << 5);
    }
    return s;
}

#[no_mangle]
pub fn apply(receiver: u64, code: u64, action: u64) {
    if (action == name_to_u64("create")) {
        action_create();
    } else if (action == name_to_u64("issue")) {
        action_issue(code);
    } else if (action == name_to_u64("transfer")) {
        action_transfer(code);
    } else {
        check(false, "unknown action");
    }
}
