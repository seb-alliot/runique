// Behaviour of static/js/csrf.js, run in isolation: no server, no browser.
// `node --test 'runique/tests/js/*.test.mjs'`
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const SCRIPT = readFileSync(new URL('../../static/js/csrf.js', import.meta.url), 'utf8');
const ORIGIN = 'https://app.example';
const TOKEN = 'page-token';

// Loads csrf.js into a fresh page. `responseToken` is what the fake network
// answers in `X-CSRF-Token`. Returns the page and every request that reached
// the network, with the headers it carried.
function page({ responseToken = null } = {}) {
    const sent = [];
    const input = { value: TOKEN };
    const ctx = {
        URL,
        Request,
        Headers,
        Response,
        HTMLFormElement: class {},
        location: { href: `${ORIGIN}/plats/1/edit`, origin: ORIGIN },
        document: {
            querySelector: (sel) => (sel === 'input[name="csrf_token"]' ? input : null),
            querySelectorAll: (sel) => (sel === 'input[name="csrf_token"]' ? [input] : []),
            addEventListener: () => {},
        },
        fetch: async (req, init = {}) => {
            sent.push({ url: req instanceof Request ? req.url : String(req), headers: new Headers(init.headers || {}) });
            const headers = responseToken ? { 'X-CSRF-Token': responseToken } : {};
            return new Response('', { headers });
        },
    };
    ctx.window = ctx;
    vm.runInNewContext(SCRIPT, ctx);
    return { win: ctx, sent, input };
}

test('a POST to the same origin carries the token', async () => {
    const { win, sent } = page();
    await win.fetch('/plats/1/edit', { method: 'POST' });
    await win.fetch(`${ORIGIN}/api`, { method: 'DELETE' });
    assert.equal(sent[0].headers.get('X-CSRF-Token'), TOKEN);
    assert.equal(sent[1].headers.get('X-CSRF-Token'), TOKEN);
});

test('a POST to another origin never carries the token', async () => {
    const { win, sent } = page();
    for (const url of [
        'https://evil.example/collect',
        '//evil.example/collect',
        'https://app.example.evil.example/',
        'http://app.example/plats', // same host, other scheme: another origin
    ]) {
        await win.fetch(url, { method: 'POST' });
    }
    await win.fetch(new Request('https://evil.example/collect'), { method: 'POST' });
    assert.equal(sent.length, 5, 'the requests still go out');
    for (const r of sent) {
        assert.equal(r.headers.get('X-CSRF-Token'), null, r.url);
    }
});

test('a GET carries no token, even to the same origin', async () => {
    const { win, sent } = page();
    await win.fetch('/plats');
    assert.equal(sent[0].headers.get('X-CSRF-Token'), null);
});

test("only the page's own origin may rotate the token", async () => {
    const other = page({ responseToken: 'planted' });
    await other.win.fetch('https://evil.example/', { method: 'POST' });
    assert.equal(other.input.value, TOKEN, 'another site cannot plant a token');

    const own = page({ responseToken: 'rotated' });
    await own.win.fetch('/plats', { method: 'POST' });
    assert.equal(own.input.value, 'rotated', 'the server rotates it as before');
});
