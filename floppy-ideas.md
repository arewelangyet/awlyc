## General Information

Each file can contains at most one singular ['value'](#Values) which will serve as the entry-point to construct the declarative tree if the parser is ran directly against that file.

In addition to the one value; a file can contain an arbitrary amount of [imports](#Imports) and [functions](#Functions)

## Values

### Arrays

```python
[VALUE, VALUE]
```

### Records

```python
{ FIELD_NAME: VALUE, OTHER_FIELD: VALUE }
```

### Literals

```python
1234
```

```python
"my string"
```

```python
1234.1234
```

## Functions

Functions take a set of values to be instantiated into their own function body as the value, but with each parameter identifier substituted for the correlated value given as parameter. 

```python
fn defaultSettings(token):
    { token: token, auth: "normal", expire_after: 500 }

{ settings: defaultSettings("1234"), host: "https://arewelangyet.com/" }
```

Would construct the following value

```python
{ settings = { token: "1234", auth: "normal", expire_after: 500 }, host: "https://arewelangyet.com/" }
```

## Imports

```python
import helper "helper_functions.awlyc"
```

Any functions declared inside the imported file can now also be used by using the qualified path `helper.myFunction()`
  
Circular imports are allowed
  
A file does not expose it's own imports. So; you cannot import a file *through* another file.

## Complete Example

```python
import parsing "pages.awlyc"
import interning "interning.awlyc"

fn host():
  "https://arewelangyet.com/"

fn linkTo(path):
  # I guess we want to provide some builtins like this one?
  concat(host(), path)

[
  {
    category: "Parsing & Lexing",
    link: linkTo("parsing"),
    pages: [
      parsing.logos(),
      parsing.lalrpop(),
      parsing.chumsky()
    ]
  },
  {
    category: "String Interning",
    link: linkTo("interning"),
    pages: [
      interning.lasso()
    ]
  }
]
```
