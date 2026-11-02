# Interpolate data

Use interpolation to write context values into the rendered output.

## Read values

Given this context:

```ts
const context = {
    customer: {
        name: "Mira Chen",
        address: {city: "Suzhou"},
    },
};
```

the template can read nested members:

```dejavu
<h1>Welcome, <% customer.name %></h1>
<p>Delivery city: <% customer.address.city %></p>
```

Expected output:

```html
<h1>Welcome, Mira Chen</h1>
<p>Delivery city: Suzhou</p>
```

List and object values can also be indexed with brackets, for example `<% orders[0].number %>`.

## Handle missing values

By default, a missing identifier renders as an empty string. During development, enable strict checks so a missing name
throws instead:

```ts
Dejavu.renderSource("Account: <% account.name %>", {}, {
    strictUndefined: true,
});
```

Use the default behavior only when an absent value is an expected presentation case. Use `strictUndefined` in tests and
build checks to catch misspelled paths.

## Escaping

Interpolation escapes HTML characters by default:

```dejavu
Message: <% message %>
```

With `{ message: "<script>alert(1)</script>" }`, the script is emitted as text. Do not bypass this behavior for
user-controlled values. The [filters guide](./filters.md) explains the explicit `safe` and `raw` escape hatches.

Continue with [conditions and loops](./control-flow.md).
