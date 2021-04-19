initSidebarItems({"attr":[["async_trait",""]],"derive":[["Outgoing","Derive the `Outgoing` trait, possibly generating an ‘Incoming’ version of the struct this derive macro is used on. Specifically, if no lifetime variables are used on any of the fields of the struct, this simple implementation will be generated:"]],"enum":[["AuthScheme","Authentication scheme used by the endpoint."],["BaseError","Internal representation of errors."],["CanonicalJsonValue",""],["CustomEvent","This represents the various “unrecognized” events."],["Error","Internal representation of errors."],["FromHttpRequestError","An error when converting a http request to one of ruma’s endpoint-specific request types."],["FromHttpResponseError","An error when converting a http response to one of Ruma’s endpoint-specific response types."],["HttpError","An HTTP error, representing either a connection error or an error while converting the raw HTTP response into a Matrix response."],["IntoHttpError","An error when converting one of ruma’s endpoint-specific request or response types to the corresponding http type."],["LocalTrust","The local trust state of a device."],["LoopCtrl","Enum controlling if a loop running callbacks should continue or abort."],["RoomType","Enum keeping track in which state the room is, e.g. if our own user is joined, invited, or has left the room."],["ServerError","An error was reported by the server (HTTP status code 4xx or 5xx)"],["StoreError","State store specific error type."]],"macro":[["assign","Mutate a struct value in a declarative style."],["int","Creates an `Int` from a numeric literal."],["uint","Creates a `UInt` from a numeric literal."]],"mod":[["api","(De)serializable types for the Matrix Client-Server API. These types can be shared by client and server code."],["deserialized_responses",""],["directory","Common types for room directory endpoints."],["encryption","Common types for encryption related tasks."],["events","(De)serializable types for the events in the Matrix specification. These types are used by other ruma crates."],["executor","Abstraction over an executor so we can spawn tasks under WASM the same way we do usually."],["instant",""],["locks",""],["presence","Common types for the presence module."],["push","Common types for the push notifications module."],["room","High-level room API"],["thirdparty","Common types for the third party networks module."],["uuid","Generate and parse UUIDs."]],"struct":[["BaseRoom","The underlying room data structure collecting state for joined, left and invtied rooms."],["BaseRoomMember","A member of a room."],["Client","An async/await enabled Matrix client."],["ClientConfig","Configuration for the creation of the `Client`."],["Device","A device represents a E2EE capable client of an user."],["EncryptionInfo","Struct holding all the information that is needed to decrypt an encrypted file."],["Int","An integer limited to the range of integers that can be represented exactly by an f64."],["Raw","A wrapper around `Box<RawValue>`, to be used in place of any type in the Matrix endpoint definition to allow request and response types to contain that said type represented by the generic argument `Ev`."],["RequestConfig","Configuration for requests the `Client` makes."],["RoomInfo","The underlying pure data structure for joined and left rooms."],["RoomMember","The high-level `RoomMember` representation"],["Sas","An object controling the interactive verification flow."],["Session","A user session, containing an access token and information about the associated user account."],["StateChanges","Store state changes and pass them to the StateStore."],["SyncSettings","Settings for a sync call."],["UInt","An integer limited to the range of non-negative integers that can be represented exactly by an f64."]],"trait":[["AsyncTraitDeps","Super trait that is used for our store traits, this trait will differ if it’s used on WASM. WASM targets will not require `Send` and `Sync` to have implemented, while other targets will."],["EndpointError","Gives users the ability to define their own serializable / deserializable errors."],["EventHandler","This trait allows any type implementing `EventHandler` to specify event callbacks for each event. The `Client` calls each method when the corresponding event is received."],["HttpSend","Abstraction around the http layer. The allows implementors to use different http libraries."],["IncomingResponse","A response type for a Matrix API endpoint, used for receiving responses."],["Outgoing","A type that can be sent to another party that understands the matrix protocol."],["OutgoingRequest","A request type for a Matrix API endpoint, used for sending requests."]],"type":[["Result","Result type of the rust-sdk."]]});