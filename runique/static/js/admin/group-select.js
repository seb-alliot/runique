// Rights page: picking another group in the selector opens its page at once.
// The form is a GET that changes nothing; without this script, its button
// does the same.
document.addEventListener("change", (event) => {
    const select = event.target.closest("select[data-autosubmit]");
    if (select) {
        select.form.submit();
    }
});
