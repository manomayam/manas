// THIS FILE IS GENERATED. ONE SHOULD NOT MODIFY IT
//! I define statics for `Solid Protocol` specification.
//!

// #![allow(clippy::needless_raw_string_hashes)]

use crate::spec_mod;

spec_mod! {
    /// Solid Protocol.
    Spec: (
        SolidProtocol,
        "Solid Protocol",
        "https://solidproject.org/ED/protocol"
    );

    Subjects: [
        (SUBJECT_CLIENT, "https://solidproject.org/ED/protocol#Client"),
        (SUBJECT_SERVER, "https://solidproject.org/ED/protocol#Server"),
    ];

    Requirements: [
        (
            REQ_SERVER_DISALLOW_DELETE,
            "https://solidproject.org/ED/protocol#server-disallow-delete",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Server MUST exclude the DELETE method in the HTTP response header Allow in response to requests to these resources [RFC7231]."###
        ),
        (
            REQ_SERVER_PATCH_N3_TYPE,
            "https://solidproject.org/ED/protocol#server-patch-n3-type",
            RequirementLevel::Must,
            [],
            r###"?patch rdf:type solid:InsertDeletePatch"###
        ),
        (
            REQ_SERVER_DELETE_PROTECT_ROOT_CONTAINER,
            "https://solidproject.org/ED/protocol#server-delete-protect-root-container",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a DELETE request targets storage’s root container or its associated ACL resource, the server MUST respond with the 405 status code."###
        ),
        (
            REQ_SERVER_N3_PATCH_DELETE,
            "https://solidproject.org/ED/protocol#server-n3-patch-delete",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When ?deletions is non-empty, servers MUST treat the request as a Read and Write operation."###
        ),
        (
            REQ_SERVER_CORS_OPTIONS,
            "https://solidproject.org/ED/protocol#server-cors-options",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"A server MUST also support the HTTP OPTIONS method [RFC7231] such that it can respond appropriately to CORS preflight requests."###
        ),
        (
            REQ_SERVER_CORS,
            "https://solidproject.org/ED/protocol#server-cors",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"A server MUST implement the CORS protocol [FETCH] such that, to the extent possible, the browser allows Solid apps to send any request and combination of request headers to the server, and the Solid app can read any response and response headers received from the server. If the server wishes to block access to a resource, this MUST NOT happen via CORS but MUST instead be communicated to the Solid app in the browser through HTTP status codes such as 401, 403, or 404 [RFC7231]."###
        ),
        (
            REQ_SERVER_PATCH_N3_ADVERTISE,
            "https://solidproject.org/ED/protocol#server-patch-n3-advertise",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST indicate support of N3 Patch by listing text/n3 as a value of the Accept-Patch header [RFC5789] of relevant responses."###
        ),
        (
            REQ_CLIENT_AUTHENTICATION,
            "https://solidproject.org/ED/protocol#client-authentication",
            RequirementLevel::Must,
            [SUBJECT_CLIENT,],
            r###"Clients MUST conform to HTTP/1.1 Authentication [RFC7235] if it needs to access resources requiring authentication (see WebID)."###
        ),
        (
            REQ_SERVER_CORS_ACAO_VARY,
            "https://solidproject.org/ED/protocol#server-cors-acao-vary",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"the server MUST set the Access-Control-Allow-Origin header to the valid Origin value from the request and list Origin in the Vary header value."###
        ),
        (
            REQ_CLIENT_HTTP_2,
            "https://solidproject.org/ED/protocol#client-http-2",
            RequirementLevel::May,
            [SUBJECT_CLIENT,],
            r###"Clients MAY conform to HTTP/2 [RFC7540]."###
        ),
        (
            REQ_SERVER_HTTP_2,
            "https://solidproject.org/ED/protocol#server-http-2",
            RequirementLevel::Should,
            [SUBJECT_SERVER,],
            r###"Servers SHOULD conform to HTTP/2 [RFC7540]."###
        ),
        (
            REQ_SERVER_CACHING,
            "https://solidproject.org/ED/protocol#server-caching",
            RequirementLevel::Should,
            [SUBJECT_SERVER,],
            r###"Servers SHOULD conform to HTTP/1.1 Caching [RFC7234]."###
        ),
        (
            REQ_SERVER_PATCH_N3_SEMANTICS_DELETIONS_NON_EMPTY_ALL_TRIPLES,
            "https://solidproject.org/ED/protocol#server-patch-n3-semantics-deletions-non-empty-all-triples",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"server MUST respond with a 409 status code."###
        ),
        (
            REQ_SERVER_ETAG,
            "https://solidproject.org/ED/protocol#server-etag",
            RequirementLevel::May,
            [SUBJECT_SERVER,],
            r###"Servers MAY use the HTTP ETag header with a strong validator for RDF bearing representations in order to encourage clients to opt-in to using the If-Match header in their requests."###
        ),
        (
            REQ_SERVER_BASIC_CONTAINER,
            "https://solidproject.org/ED/protocol#server-basic-container",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"The representation and behaviour of containers in Solid corresponds to LDP Basic Container and MUST be supported by server."###
        ),
        (
            REQ_SERVER_CONDITIONAL_REQUESTS,
            "https://solidproject.org/ED/protocol#server-conditional-requests",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST conform to HTTP/1.1 Conditional Requests [RFC7232]."###
        ),
        (
            REQ_SERVER_PATCH_N3_PATCH_IDENTIFIER,
            "https://solidproject.org/ED/protocol#server-patch-n3-patch-identifier",
            RequirementLevel::Must,
            [],
            r###"A patch resource MUST be identified by a URI or blank node, which we refer to as ?patch in the remainder of this section."###
        ),
        (
            REQ_SERVER_NOTIFICATIONS_PROTOCOL_RESOURCE_SERVER,
            "https://solidproject.org/ED/protocol#server-notifications-protocol-resource-server",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST conform to the Solid Notifications Protocol [SOLID-NOTIFICATIONS-PROTOCOL] by implementing the Resource Server to enable clients to discover subscription resources and notification channels available to a given resource or storage."###
        ),
        (
            REQ_SERVER_SLUG_URI_ASSIGNMENT,
            "https://solidproject.org/ED/protocol#server-slug-uri-assignment",
            RequirementLevel::May,
            [SUBJECT_SERVER,],
            r###"Servers MAY allow clients to suggest the URI of a resource created through POST, using the HTTP Slug header as defined in [RFC5023]."###
        ),
        (
            REQ_CLIENT_ACP,
            "https://solidproject.org/ED/protocol#client-acp",
            RequirementLevel::Must,
            [SUBJECT_CLIENT,],
            r###"Clients MUST conform to the Access Control Policy specification [ACP]."###
        ),
        (
            REQ_SERVER_AUXILIARY_RESOURCES_MANAGEMENT,
            "https://solidproject.org/ED/protocol#server-auxiliary-resources-management",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST support auxiliary resources defined by this specification and manage the association between a subject resource and auxiliary resources."###
        ),
        (
            REQ_SERVER_CONTENT_TYPE_PAYLOAD,
            "https://solidproject.org/ED/protocol#server-content-type-payload",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Server MUST generate a Content-Type header field in a message that contains a payload body."###
        ),
        (
            REQ_SERVER_CORS_ACEH,
            "https://solidproject.org/ED/protocol#server-cors-aceh",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"The server MUST make all used response headers readable for the Solid app through Access-Control-Expose-Headers (with the possible exception of the Access-Control-* headers themselves)."###
        ),
        (
            REQ_CLIENT_HTTP_11,
            "https://solidproject.org/ED/protocol#client-http-11",
            RequirementLevel::Must,
            [SUBJECT_CLIENT,],
            r###"Clients MUST conform to HTTP/1.1 Message Syntax and Routing [RFC7230] and HTTP/1.1 Semantics and Content [RFC7231]."###
        ),
        (
            REQ_SERVER_LINK_STORAGE,
            "https://solidproject.org/ED/protocol#server-link-storage",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST advertise the storage resource by including the HTTP Link header with rel="type" targeting http://www.w3.org/ns/pim/space#Storage when responding to storage’s request URI."###
        ),
        (
            REQ_SERVER_STORAGE_TRACK_OWNER,
            "https://solidproject.org/ED/protocol#server-storage-track-owner",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST keep track of at least one owner of a storage in an implementation defined way."###
        ),
        (
            REQ_SERVER_PATCH_N3_INVALID,
            "https://solidproject.org/ED/protocol#server-patch-n3-invalid",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST respond with a 422 status code [RFC4918] if a patch document does not satisfy all of the above constraints."###
        ),
        (
            REQ_SERVER_N3_PATCH_WHERE,
            "https://solidproject.org/ED/protocol#server-n3-patch-where",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When ?conditions is non-empty, servers MUST treat the request as a Read operation."###
        ),
        (
            REQ_SERVER_STORAGE_LINK_OWNER,
            "https://solidproject.org/ED/protocol#server-storage-link-owner",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a server wants to advertise the owner of a storage, the server MUST include the Link header with rel="http://www.w3.org/ns/solid/terms#owner" targeting the URI of the owner in the response of HTTP HEAD or GET requests targeting the root container."###
        ),
        (
            REQ_SERVER_PATCH_N3_SEMANTICS,
            "https://solidproject.org/ED/protocol#server-patch-n3-semantics",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST process a patch resource against the target document as follows:"###
        ),
        (
            REQ_CLIENT_CACHING,
            "https://solidproject.org/ED/protocol#client-caching",
            RequirementLevel::May,
            [SUBJECT_CLIENT,],
            r###"Clients MAY conform to HTTP/1.1 Caching [RFC7234]."###
        ),
        (
            REQ_SERVER_DESCRIPTION_RESOURCE_MAX,
            "https://solidproject.org/ED/protocol#server-description-resource-max",
            RequirementLevel::MustNot,
            [SUBJECT_SERVER,],
            r###"Servers MUST NOT directly associate more than one description resource to a subject resource."###
        ),
        (
            REQ_SERVER_POST_CONTAINER_CREATE_RESOURCE,
            "https://solidproject.org/ED/protocol#server-post-container-create-resource",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST create a resource with URI path ending /{id} in container /."###
        ),
        (
            REQ_SERVER_PATCH_N3_SINGLE,
            "https://solidproject.org/ED/protocol#server-patch-n3-single",
            RequirementLevel::Must,
            [],
            r###"The patch document MUST contain exactly one patch resource, identified by one or more of the triple patterns described above, which all share the same ?patch subject."###
        ),
        (
            REQ_SERVER_PROTECT_CONTAINMENT,
            "https://solidproject.org/ED/protocol#server-protect-containment",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST NOT allow HTTP PUT or PATCH on a container to update its containment triples; if the server receives such a request, it MUST respond with a 409 status code."###
        ),
        (
            REQ_SERVER_STORAGE_NONOVERLAPPING,
            "https://solidproject.org/ED/protocol#server-storage-nonoverlapping",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a server supports multiple storages, the URIs MUST be allocated to non-overlapping space."###
        ),
        (
            REQ_SERVER_NOTIFICATIONS_PROTOCOL_NOTIFICATION_RECEIVER,
            "https://solidproject.org/ED/protocol#server-notifications-protocol-notification-receiver",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST conform to the Solid Notifications Protocol [SOLID-NOTIFICATIONS-PROTOCOL] by implementing the Notification Receiver to receive and process messages that conform to a notification channel type."###
        ),
        (
            REQ_SERVER_POST_CONTAINER_CREATE_CONTAINER,
            "https://solidproject.org/ED/protocol#server-post-container-create-container",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST create a container with URI path ending /{id}/ in container / for requests including the HTTP Link header with rel="type" targeting a valid LDP container type."###
        ),
        (
            REQ_SERVER_STORAGE_DESCRIPTION_RESOURCE,
            "https://solidproject.org/ED/protocol#server-storage-description-resource",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST include statements about the storage as part of the storage description resource."###
        ),
        (
            REQ_SERVER_DESCRIPTION_RESOURCE_AUTHORIZATION,
            "https://solidproject.org/ED/protocol#server-description-resource-authorization",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When an HTTP request targets a description resource, the server MUST apply the authorization rule that is used for the subject resource with which the description resource is associated."###
        ),
        (
            REQ_SERVER_AUTHORIZATION_REDIRECT,
            "https://solidproject.org/ED/protocol#server-authorization-redirect",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST authorize prior to this optional redirect."###
        ),
        (
            REQ_SERVER_POST_CONTAINER,
            "https://solidproject.org/ED/protocol#server-post-container",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST allow creating new resources with a POST request to URI path ending /."###
        ),
        (
            REQ_CLIENT_LDN,
            "https://solidproject.org/ED/protocol#client-ldn",
            RequirementLevel::Must,
            [SUBJECT_CLIENT,],
            r###"A Solid client MUST conform to the LDN specification by implementing the Sender or Consumer parts to discover the location of a resource’s Inbox, and to send notifications to an Inbox or to retrieve the contents of an Inbox [LDN]."###
        ),
        (
            REQ_SERVER_PATCH_N3_WHERE,
            "https://solidproject.org/ED/protocol#server-patch-n3-where",
            RequirementLevel::Must,
            [],
            r###"A patch resource MUST contain at most one triple of the form ?patch solid:where ?conditions."###
        ),
        (
            REQ_CLIENT_CONDITIONAL_REQUESTS,
            "https://solidproject.org/ED/protocol#client-conditional-requests",
            RequirementLevel::May,
            [SUBJECT_CLIENT,],
            r###"Clients MAY conform to HTTP/1.1 Conditional Requests [RFC7232]."###
        ),
        (
            REQ_SERVER_HTTP_11,
            "https://solidproject.org/ED/protocol#server-http-11",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST conform to HTTP/1.1 Message Syntax and Routing [RFC7230] and HTTP/1.1 Semantics and Content [RFC7231]."###
        ),
        (
            REQ_SERVER_POST_SLUG_AUXILIARY_RESOURCE,
            "https://solidproject.org/ED/protocol#server-post-slug-auxiliary-resource",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a POST method request with the Slug header targets an auxiliary resource, the server MUST respond with the 403 status code and response body describing the error."###
        ),
        (
            REQ_SERVER_PUT_PATCH_URI_ASSIGNMENT,
            "https://solidproject.org/ED/protocol#server-put-patch-uri-assignment",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a successful PUT or PATCH request creates a resource, the server MUST use the effective request URI to assign the URI to that resource."###
        ),
        (
            REQ_SERVER_TLS_HTTPS,
            "https://solidproject.org/ED/protocol#server-tls-https",
            RequirementLevel::Should,
            [SUBJECT_SERVER,],
            r###"Servers SHOULD use TLS connections through the https URI scheme in order to secure the communication with clients."###
        ),
        (
            REQ_SERVER_URI_REDIRECT_DIFFERING,
            "https://solidproject.org/ED/protocol#server-uri-redirect-differing",
            RequirementLevel::May,
            [SUBJECT_SERVER,],
            r###"Instead, the server MAY respond to requests for the latter URI with a 301 redirect to the former."###
        ),
        (
            REQ_SERVER_REPRESENTATION_WRITE_REDIRECT,
            "https://solidproject.org/ED/protocol#server-representation-write-redirect",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a PUT, POST, PATCH or DELETE method request targets a representation URL that is different than the resource URL, the server MUST respond with a 307 or 308 status code and Location header specifying the preferred URI reference."###
        ),
        (
            REQ_SERVER_STORAGE_DESCRIPTION,
            "https://solidproject.org/ED/protocol#server-storage-description",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST include the Link header with rel="http://www.w3.org/ns/solid/terms#storageDescription" targeting the URI of the storage description resource in the response of HTTP GET, HEAD and OPTIONS requests targeting a resource in a storage."###
        ),
        (
            REQ_SERVER_LDN,
            "https://solidproject.org/ED/protocol#server-ldn",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"A Solid server MUST conform to the LDN specification by implementing the Receiver parts to receive notifications and make Inbox contents available [LDN]."###
        ),
        (
            REQ_SERVER_PATCH_N3_PATCHES,
            "https://solidproject.org/ED/protocol#server-patch-n3-patches",
            RequirementLevel::Must,
            [],
            r###"A patch document MUST contain one or more patch resources."###
        ),
        (
            REQ_SERVER_TLS_HTTPS_REDIRECT,
            "https://solidproject.org/ED/protocol#server-tls-https-redirect",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When both http and https URI schemes are supported, the server MUST redirect all http URIs to their https counterparts using a response with a 301 status code and a Location header."###
        ),
        (
            REQ_SERVER_RANGE_REQUESTS,
            "https://solidproject.org/ED/protocol#server-range-requests",
            RequirementLevel::May,
            [SUBJECT_SERVER,],
            r###"Servers MAY conform to HTTP/1.1 Range Requests [RFC7233]."###
        ),
        (
            REQ_SERVER_ALLOW_METHODS,
            "https://solidproject.org/ED/protocol#server-allow-methods",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST indicate the HTTP methods supported by the target resource by generating an Allow header field in successful responses."###
        ),
        (
            REQ_SERVER_PROTECT_CONTAINED_RESOURCE_METADATA,
            "https://solidproject.org/ED/protocol#server-protect-contained-resource-metadata",
            RequirementLevel::MustNot,
            [SUBJECT_SERVER,],
            r###"Servers MUST NOT allow HTTP POST, PUT and PATCH to update a container’s resource metadata statements; if the server receives such a request, it MUST respond with a 409 status code."###
        ),
        (
            REQ_SERVER_AUTHENTICATION,
            "https://solidproject.org/ED/protocol#server-authentication",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST conform to HTTP/1.1 Authentication [RFC7235]."###
        ),
        (
            REQ_SERVER_PATCH_N3_FORMULAE,
            "https://solidproject.org/ED/protocol#server-patch-n3-formulae",
            RequirementLevel::Must,
            [],
            r###"When present, ?deletions, ?insertions, and ?conditions MUST be non-nested cited formulae [N3] consisting only of triples and/or triple patterns [SPARQL11-QUERY]. When not present, they are presumed to be the empty formula {}."###
        ),
        (
            REQ_SERVER_CORS_ACCESS_CONTROL_HEADERS,
            "https://solidproject.org/ED/protocol#server-cors-access-control-headers",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"whenever a server receives an HTTP request containing a valid Origin header [RFC6454], the server MUST respond with the appropriate Access-Control-* headers as specified in the CORS protocol [FETCH]."###
        ),
        (
            REQ_SERVER_PUT_PATCH_AUXILIARY_RESOURCE,
            "https://solidproject.org/ED/protocol#server-put-patch-auxiliary-resource",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a PUT or PATCH method request targets an auxiliary resource, the server MUST create or update it."###
        ),
        (
            REQ_SERVER_PATCH_N3_DELETES,
            "https://solidproject.org/ED/protocol#server-patch-n3-deletes",
            RequirementLevel::Must,
            [],
            r###"A patch resource MUST contain at most one triple of the form ?patch solid:deletes ?deletions."###
        ),
        (
            REQ_SERVER_UNAUTHENTICATED,
            "https://solidproject.org/ED/protocol#server-unauthenticated",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a client does not provide valid credentials when requesting a resource that requires it (see WebID), servers MUST send a response with a 401 status code (unless 404 is preferred for security reasons)."###
        ),
        (
            REQ_CLIENT_WAC,
            "https://solidproject.org/ED/protocol#client-wac",
            RequirementLevel::Must,
            [SUBJECT_CLIENT,],
            r###"Clients MUST conform to the Web Access Control specification [WAC]."###
        ),
        (
            REQ_SERVER_CONTENT_TYPE,
            "https://solidproject.org/ED/protocol#server-content-type",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Server MUST reject PUT, POST and PATCH requests without the Content-Type header with a status code of 400."###
        ),
        (
            REQ_SERVER_ACCEPT_HEADERS,
            "https://solidproject.org/ED/protocol#server-accept-headers",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When responding to authorized requests, servers MUST indicate supported media types in the HTTP Accept-Patch [RFC5789], Accept-Post [LDP] and Accept-Put [The Accept-Put Response Header] response headers that correspond to acceptable HTTP methods listed in Allow header value in response to HTTP GET, HEAD and OPTIONS requests."###
        ),
        (
            REQ_CLIENT_RANGE_REQUESTS,
            "https://solidproject.org/ED/protocol#client-range-requests",
            RequirementLevel::May,
            [SUBJECT_CLIENT,],
            r###"Clients MAY conform to HTTP/1.1 Range Requests [RFC7233]."###
        ),
        (
            REQ_SERVER_CORS_ENUMERATE,
            "https://solidproject.org/ED/protocol#server-cors-enumerate",
            RequirementLevel::Should,
            [SUBJECT_SERVER,],
            r###"servers SHOULD explicitly enumerate all used response headers under Access-Control-Expose-Headers rather than resorting to *, which does not cover all cases (such as credentials mode set to include)."###
        ),
        (
            REQ_SERVER_DELETE_REMOVE_EMPTY_CONTAINER,
            "https://solidproject.org/ED/protocol#server-delete-remove-empty-container",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a DELETE request targets a container, the server MUST delete the container if it contains no resources."###
        ),
        (
            REQ_SERVER_PUT_PATCH_INTERMEDIATE_CONTAINERS,
            "https://solidproject.org/ED/protocol#server-put-patch-intermediate-containers",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST create intermediate containers and include corresponding containment triples in container representations derived from the URI path component of PUT and PATCH requests."###
        ),
        (
            REQ_SERVER_DELETE_PROTECT_NONEMPTY_CONTAINER,
            "https://solidproject.org/ED/protocol#server-delete-protect-nonempty-container",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"If the container contains resources, the server MUST respond with the 409 status code and response body describing the error."###
        ),
        (
            REQ_SERVER_NOTIFICATIONS_PROTOCOL_SUBSCRIPTION_SERVER,
            "https://solidproject.org/ED/protocol#server-notifications-protocol-subscription-server",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST conform to the Solid Notifications Protocol [SOLID-NOTIFICATIONS-PROTOCOL] by implementing the Subscription Server to process and produce instructions for subscription requests."###
        ),
        (
            REQ_SERVER_WAC_ACP,
            "https://solidproject.org/ED/protocol#server-wac-acp",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST conform to either or both Web Access Control [WAC] and Access Control Policy [ACP] specifications."###
        ),
        (
            REQ_SERVER_URI_TRAILING_SLASH_DISTINCT,
            "https://solidproject.org/ED/protocol#server-uri-trailing-slash-distinct",
            RequirementLevel::MustNot,
            [SUBJECT_SERVER,],
            r###"If two URIs differ only in the trailing slash, and the server has associated a resource with one of them, then the other URI MUST NOT correspond to another resource."###
        ),
        (
            REQ_SERVER_CORS_ACCEPT_ACAH,
            "https://solidproject.org/ED/protocol#server-cors-accept-acah",
            RequirementLevel::Should,
            [SUBJECT_SERVER,],
            r###"Servers SHOULD also explicitly list Accept under Access-Control-Allow-Headers"###
        ),
        (
            REQ_SERVER_N3_PATCH_INSERT,
            "https://solidproject.org/ED/protocol#server-n3-patch-insert",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When ?insertions is non-empty, servers MUST (also) treat the request as an Append operation."###
        ),
        (
            REQ_SERVER_DELETE_REMOVE_AUXILIARY_RESOURCE,
            "https://solidproject.org/ED/protocol#server-delete-remove-auxiliary-resource",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a contained resource is deleted, the server MUST also delete the associated auxiliary resources (see the Auxiliary Resources section)."###
        ),
        (
            REQ_SERVER_PATCH_N3_BLANK_NODES,
            "https://solidproject.org/ED/protocol#server-patch-n3-blank-nodes",
            RequirementLevel::MustNot,
            [],
            r###"The ?insertions and ?deletions formulae MUST NOT contain blank nodes."###
        ),
        (
            REQ_SERVER_REPRESENTATION_TURTLE_JSONLD,
            "https://solidproject.org/ED/protocol#server-representation-turtle-jsonld",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a server creates a resource on HTTP PUT, POST or PATCH requests such that the request’s representation data encodes an RDF document [RDF11-CONCEPTS] (as determined by the Content-Type header), the server MUST accept GET requests on this resource when the value of the Accept header requests a representation in text/turtle or application/ld+json [Turtle] [JSON-LD11]."###
        ),
        (
            REQ_CLIENT_CONTENT_TYPE,
            "https://solidproject.org/ED/protocol#client-content-type",
            RequirementLevel::Must,
            [SUBJECT_CLIENT,],
            r###"Clients MUST use the Content-Type HTTP header in PUT, POST and PATCH requests [RFC7231]."###
        ),
        (
            REQ_SERVER_CONTAINED_RESOURCE_METADATA,
            "https://solidproject.org/ED/protocol#server-contained-resource-metadata",
            RequirementLevel::Should,
            [SUBJECT_SERVER,],
            r###"Servers SHOULD include resource metadata about contained resources as part of the container description, unless that information is inapplicable to the server."###
        ),
        (
            REQ_SERVER_DELETE_REMOVE_CONTAINMENT,
            "https://solidproject.org/ED/protocol#server-delete-remove-containment",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a contained resource is deleted, the server MUST also remove the corresponding containment triple."###
        ),
        (
            REQ_SERVER_POST_TARGET_NOT_FOUND,
            "https://solidproject.org/ED/protocol#server-post-target-not-found",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a POST method request targets a resource without an existing representation, the server MUST respond with the 404 status code."###
        ),
        (
            REQ_CLIENT_AUTHENTICATION_DIFFERENT_CREDENTIALS,
            "https://solidproject.org/ED/protocol#client-authentication-different-credentials",
            RequirementLevel::May,
            [SUBJECT_CLIENT,],
            r###"When a client receives a response with a 403 or 404 status code, the client MAY repeat the request with different credentials."###
        ),
        (
            REQ_SERVER_PATCH_N3_SEMANTICS_NO_MAPPING,
            "https://solidproject.org/ED/protocol#server-patch-n3-semantics-no-mapping",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"server MUST respond with a 409 status code."###
        ),
        (
            REQ_SERVER_POST_URI_ASSIGNMENT,
            "https://solidproject.org/ED/protocol#server-post-uri-assignment",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"When a successful POST request creates a resource, the server MUST assign a URI to that resource."###
        ),
        (
            REQ_SERVER_PATCH_N3_VARIABLES,
            "https://solidproject.org/ED/protocol#server-patch-n3-variables",
            RequirementLevel::MustNot,
            [],
            r###"The ?insertions and ?deletions formulae MUST NOT contain variables that do not occur in the ?conditions formula."###
        ),
        (
            REQ_SERVER_NOTIFICATIONS_PROTOCOL_NOTIFICATION_SENDER,
            "https://solidproject.org/ED/protocol#server-notifications-protocol-notification-sender",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST conform to the Solid Notifications Protocol [SOLID-NOTIFICATIONS-PROTOCOL] by implementing the Notification Sender to produce and send messages to a Notification Receiver."###
        ),
        (
            REQ_SERVER_PATCH_N3_ACCEPT,
            "https://solidproject.org/ED/protocol#server-patch-n3-accept",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST accept a PATCH request with an N3 Patch body when the target of the request is an RDF document [RDF11-CONCEPTS]."###
        ),
        (
            REQ_SERVER_STORAGE,
            "https://solidproject.org/ED/protocol#server-storage",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST provide one or more storages. The storage resource (pim:Storage) is the root container for all of its contained resources (see Resource Containment)."###
        ),
        (
            REQ_SERVER_METHOD_NOT_ALLOWED,
            "https://solidproject.org/ED/protocol#server-method-not-allowed",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST respond with the 405 status code to requests using HTTP methods that are not supported by the target resource."###
        ),
        (
            REQ_SERVER_LINK_AUXILIARY_TYPE,
            "https://solidproject.org/ED/protocol#server-link-auxiliary-type",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST advertise auxiliary resources associated with a subject resource by responding to HEAD and GET requests by including the HTTP Link header with the rel parameter [RFC8288]."###
        ),
        (
            REQ_SERVER_SAFE_METHODS,
            "https://solidproject.org/ED/protocol#server-safe-methods",
            RequirementLevel::Must,
            [SUBJECT_SERVER,],
            r###"Servers MUST support the HTTP GET, HEAD and OPTIONS methods [RFC7231] for clients to read resources or to determine communication options."###
        ),
    ];
}
