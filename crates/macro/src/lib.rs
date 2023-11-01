//! Various neat macros for the Rjacraft crates.

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod chat;
mod id;
mod protocol_type;

/// Construct recursive Minecraft chat components with a compact syntax.
///
/// # Environment
///
/// This macro expects that `::rjacraft_protocol` is available.
///
/// # Example
///
/// ```ignore
/// let status = chat!(
///     ("Example: " (b "basic server"))
///     ("\nYour IP: " (b,c["#22ff22"] "{}", peer.addr.ip()))
/// );
/// ```
#[proc_macro]
pub fn chat(input: TokenStream) -> TokenStream {
    chat::handle(parse_macro_input!(input)).into()
}

/// Construct Minecraft identifiers with a compile-time guarantee about their validity.
///
/// # Environment
///
/// This macro expects that `::rjacraft_protocol` is available.
///
/// # Example
///
/// ```ignore
/// assert_eq!(id!("book"), id!("minecraft:book"))
/// ```
#[proc_macro]
pub fn id(input: TokenStream) -> TokenStream {
    id::handle(parse_macro_input!(input)).into()
}

/// Nativaly represent any protocol structure in a few lines of code.
///
/// # Modes of operation
///
/// - Struct: Handles each field sequentially.
/// - Bitfield: Activated if there's a `#[bitfield]` attribute from `bitfield_struct`. Represents a
///   bitfield.
/// - Enum: Handles the discriminator (`#[variant()]`) first, then chooses a variant and
///   optionally handles its fields.
///
/// # Environment
///
/// This macro expects that `rjacraft_protocol::{error, ProtocolType}` is imported.
///
/// # Example
///
/// ```ignore
/// #[derive(Debug, Clone, ProtocolType)]
/// #[variant(VarInt<i32>)]
/// pub enum LoginPacket {
///     #[variant(0x00)]
///     LoginStart {
///         username: LenString<16>,
///         uuid: Uuid,
///     },
///
///     #[variant(0x01)]
///     EncryptionResponse {
///         shared_secret: LenVec<u8>,
///         verify_token: LenVec<u8>,
///     },
///
///     #[variant(0x02)]
///     LoginPluginResponse {
///         message_id: VarInt<i32>,
///         successful: Primitive<bool>,
///         data: RemainingBytes<{ 1 << 20 }>,
///     },
///
///     #[variant(0x03)]
///     SuccessAck,
/// }
/// ```
#[proc_macro_derive(ProtocolType, attributes(variant))]
pub fn protocol_type(item: TokenStream) -> TokenStream {
    protocol_type::handle(parse_macro_input!(item)).into()
}
