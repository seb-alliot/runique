// Loaded in <head>, before the stylesheets and without `defer`: the saved
// theme is set before anything is painted, or a light page first shows dark.
// theme.js, at the end of the body, handles the button.
try {
    if (localStorage.getItem('theme') === 'light') {
        document.documentElement.setAttribute('data-theme', 'light');
    }
} catch (e) {
    // Storage refused (private mode, blocked site data): the page stays dark.
}
