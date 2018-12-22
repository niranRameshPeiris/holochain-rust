use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use holochain_wasm_utils::api_serialization::{QueryArgs, QueryArgsNames};
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::query function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: ?
/// Returns an HcApiReturnCode as I32
///
/// Specify 0 or more simple or "glob" patterns matching EntryType names.
///
/// The empty String or an empty Vec matches all.  The '*' glob pattern matches all simple EntryType
/// names (with no '/'), while the ** pattern matches everything (use "" or [] for efficiency).
///  
/// // [ ]
/// // [ "" ]
/// // [ "**" ]
///
/// Namespaces (groups of related EntryType names) can be queried easily, eg:
///
/// // [ "name/*" ]
///
/// Several simple names and/or "glob" patterns can be supplied, and are efficiently
/// searched for in a single pass using a single efficient Regular Expression engine:
///
/// // [ "name/*", "and_another", "SomethingElse" ]
///
/// EntryType names can be excluded, eg. to return every simple (non-namespaced) EntryType except System:
///
/// // [ "[!%]*" ]
///
/// To match a pattern, including all namespaced EntryType names, eg. every EntryType except System:
///
/// // [ "**/[!%]*" ]
///
/// The following standard "glob" patterns are supported:
///
/// // Pattern     Match
/// // =======     =====
/// // .           One character (other than a '/')
/// // [abcd]      One of a set of characters
/// // [a-d]       One of a range of characters
/// // [!a-d]      Not one of  range of characters
/// // {abc,123}   One of a number of sequences of characters
/// // *           Zero or more of any character
/// // **/         Zero or more of any namespace component
///
pub fn invoke_query(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let query = match QueryArgs::try_from(args_str) {
        Ok(input) => input,
        Err(..) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };

    // Perform query
    let agent = runtime.context.state().unwrap().agent();
    let top = agent
        .top_chain_header()
        .expect("Should have genesis entries.");
    let addresses = match query.entry_type_names {
        QueryArgsNames::QueryList(pats) => {
            let refs: Vec<&str> = pats.iter().map(AsRef::as_ref).collect(); // Vec<String> -> Vec<&str>
            agent.chain().query(
                &Some(top),
                refs.as_slice(), // Vec<&str> -> Vec[&str]
                query.start,
                query.limit,
            )
        }
        QueryArgsNames::QueryName(name) => {
            let refs: Vec<&str> = vec![&name]; // String -> Vec<&str>
            agent.chain().query(
                &Some(top),
                refs.as_slice(), // Vec<&str> -> &[&str]
                query.start,
                query.limit,
            )
        }
    };
    let result = match addresses {
        // TODO #793: the Err(_code) is the RibosomeErrorCode, but we can't import that type here.
        // Perhaps return chain().query should return Some(result)/None instead, and the fixed
        // UnknownEntryType code here, rather than trying to return a specific error code.
        Ok(result) => Ok(result),
        Err(_code) => return ribosome_error_code!(UnknownEntryType),
    };

    runtime.store_result(result)
}
