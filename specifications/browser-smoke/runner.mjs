/**
 * Shared fixture runner for the browser smoke harness.
 *
 * Both the document script and the module worker import this file. It only
 * fetches fixture resources and compares rendered output against
 * `expected.out.txt`, so the two environments exercise the exact same code.
 */
export async function fetchText(url) {
    const response = await fetch(url);
    if (!response.ok) throw new Error(`fetch ${url} failed: ${response.status} ${response.statusText}`);
    return response.text();
}

export async function runFixtures(fixtures, api) {
    const results = [];
    for (const fixture of fixtures) {
        try {
            const context = JSON.parse(await fetchText(fixture.ctxUrl));
            const expected = await fetchText(fixture.expectedUrl);
            const rendered =
                fixture.kind === "source"
                    ? api.renderSource({ source: await fetchText(fixture.sourceUrl), context })
                    : api.renderIr({ ir: await fetchText(fixture.irUrl), context });
            const output = rendered.output ?? "";
            const diagnostics = rendered.diagnostics ?? [];
            results.push({
                name: fixture.name,
                kind: fixture.kind,
                ok: output === expected && diagnostics.length === 0,
                expected,
                output,
                diagnostics,
            });
        } catch (error) {
            results.push({
                name: fixture.name,
                kind: fixture.kind,
                ok: false,
                error: String((error && error.stack) || error),
            });
        }
    }
    return results;
}
