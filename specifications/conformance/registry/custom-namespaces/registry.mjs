/**
 * Host-supplied implementations for the `registry` conformance fixture.
 *
 * Export names match the `CustomRegistry` namespaces the AOT runtime reads.
 * `function` is a reserved word, so it is exported through a string literal
 * module export name (`export { function_ as "function" }`).
 */

const filter = {
    shout: (value) => `${value}!`,
    bolder: (value) => `<b>${value}</b>`,
};

const function_ = {
    twice: (args) => Number(args[0]) * 2,
};

const test = {
    evenish: (value) => Number(value) % 2 === 0,
};

export { filter, function_ as "function", test };
