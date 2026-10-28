// Rights page: the table menu (`.admin-rights__menu`, shown below 1200 px)
// closes once a table is picked, so the grid isn't left under it. Without
// this script the menu still works: its button closes it.
document.addEventListener("click", (event) => {
    const link = event.target.closest(".admin-rights__menu a");
    if (link) {
        link.closest("details").open = false;
    }
});
