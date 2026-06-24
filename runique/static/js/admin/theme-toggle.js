// Bascule sombre/clair de l'admin. data-theme est deja pose sur <html> par le
// script anti-FOUC inline ; ici on ne fait que reagir au clic et persister le choix.
(function () {
    "use strict";

    var KEY = "runique-admin-theme";
    var btn = document.getElementById("themeToggle");
    if (!btn) {
        return;
    }

    function stored() {
        try {
            var t = localStorage.getItem(KEY);
            return t === "light" || t === "dark" ? t : null;
        } catch (e) {
            return null;
        }
    }

    function current() {
        return document.documentElement.getAttribute("data-theme") === "light" ? "light" : "dark";
    }

    function apply(theme) {
        document.documentElement.setAttribute("data-theme", theme);
        btn.setAttribute("aria-pressed", theme === "light" ? "true" : "false");
        try {
            localStorage.setItem(KEY, theme);
        } catch (e) {
            // localStorage indisponible (mode prive) : la bascule reste valable pour la session.
        }
    }

    // Filet de securite si l'anti-FOUC inline a ete bloque (CSP custom sans nonce) :
    // on reapplique le choix stocke. Sinon on entherine l'etat deja pose sur <html>.
    apply(stored() || current());

    btn.addEventListener("click", function () {
        apply(current() === "dark" ? "light" : "dark");
    });
})();
