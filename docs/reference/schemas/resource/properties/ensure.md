# DSC Resource _ensure property schema

## Synopsis

Indicates whether an instance should exist.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/properties/ensure.json
Type           : string
Valid Values   : [Absent, Present]
```

## Description

The `_ensure` property indicates that the resource can enforce whether instances exist using the
shared present and absent semantics.

When `_ensure` is set to `Present` for the desired state, the instance is expected to exist. If it
doesn't exist, the resource creates the instance during the set operation.

When `_ensure` is set to `Absent` for the desired state, the instance is expected to not exist. If
it does exist, the resource removes the instance during the set operation.

To add this property to a resource's instance schema, define the property with the following
snippet:

```json
"_ensure": {
  "$ref": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/properties/ensure.json"
}
```

If a resource must distinguish between states beyond whether an instance is present or absent, the
resource should define its own `ensure` property without the leading underscore.

For example, a resource that manages a file might be designed to ensure whether the file exists, is
specifically a file, or exists as a symlink. In that case, the resource would define its own
`ensure` property, such as with the snippet below.

```json
"ensure": {
    "type": "string",
    "enum": ["present", "absent", "file", "symlink"],
    "default": "present"
}
```