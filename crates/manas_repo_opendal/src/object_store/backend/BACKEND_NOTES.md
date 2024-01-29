An opendal service requires following functionalities to be usable as an object store backend.

read (
    native_content_type: recommended,
    range_support: required,
    last_modified: required,
    etag: recommended,
): required

write (
    native_content_type: recommended,
    streaming: required,
    write_empty: required,
    abortable: required.
)

list (
    immediate_children_only: required,
    recursive: not-used,
)



S3, Fs, GCS


WebDAV:
    native_content_type: y
    range: y
    
