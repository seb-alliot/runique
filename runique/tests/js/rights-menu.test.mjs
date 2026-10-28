// Behaviour of static/js/admin/rights-menu.js, run in isolation: no server, no browser.
// `node --test 'runique/tests/js/*.test.mjs'`
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const SCRIPT = readFileSync(new URL('../../static/js/admin/rights-menu.js', import.meta.url), 'utf8');

// Loads the script into a fresh page with an open menu. Returns the menu and
// a `click(inMenu)` that fires a click on one of its links, or elsewhere.
function page() {
    const menu = { open: true };
    const link = { closest: (selector) => (selector === 'details' ? menu : null) };
    let onClick = null;
    const document = {
        addEventListener: (type, handler) => {
            if (type === 'click') onClick = handler;
        },
    };
    vm.runInNewContext(SCRIPT, { document });
    const click = (inMenu) =>
        onClick({
            target: {
                closest: (selector) =>
                    selector === '.admin-rights__menu a' && inMenu ? link : null,
            },
        });
    return { menu, click };
}

test('picking a table closes the menu', () => {
    const { menu, click } = page();
    click(true);
    assert.equal(menu.open, false);
});

test('a click elsewhere leaves the menu open', () => {
    const { menu, click } = page();
    click(false);
    assert.equal(menu.open, true);
});
