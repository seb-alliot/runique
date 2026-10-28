// Behaviour of static/js/admin/group-select.js, run in isolation: no server, no browser.
// `node --test 'runique/tests/js/*.test.mjs'`
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const SCRIPT = readFileSync(new URL('../../static/js/admin/group-select.js', import.meta.url), 'utf8');

// Loads the script into a fresh page. Returns `change(onSelector)` and how
// many times the selector's form was sent.
function page() {
    let sent = 0;
    const select = { form: { submit: () => { sent += 1; } } };
    let onChange = null;
    const document = {
        addEventListener: (type, handler) => {
            if (type === 'change') onChange = handler;
        },
    };
    vm.runInNewContext(SCRIPT, { document });
    const change = (onSelector) =>
        onChange({
            target: {
                closest: (selector) =>
                    selector === 'select[data-autosubmit]' && onSelector ? select : null,
            },
        });
    return { change, sent: () => sent };
}

test('picking a group sends the selector', () => {
    const { change, sent } = page();
    change(true);
    assert.equal(sent(), 1);
});

test('another change sends nothing', () => {
    const { change, sent } = page();
    change(false);
    assert.equal(sent(), 0);
});
