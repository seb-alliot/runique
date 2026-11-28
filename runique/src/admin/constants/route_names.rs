//! Names of the admin's generated routes, as `{% link %}` takes them:
//! `{% link "admin_detail" resource=…, id=… %}`.
/// `{prefix}/{resource}/list`
pub const LIST: &str = "admin_list";
/// `{prefix}/{resource}/{id}/detail`
pub const DETAIL: &str = "admin_detail";
/// `{prefix}/{resource}/create`
pub const CREATE: &str = "admin_create";
/// `{prefix}/{resource}/{id}/edit`
pub const EDIT: &str = "admin_edit";
/// `{prefix}/{resource}/{id}/delete`
pub const DELETE: &str = "admin_delete";
/// `{prefix}/{resource}/{id}/reset-password` (POST): sends the account's
/// owner a link to choose a new password
pub const RESET_PASSWORD: &str = "admin_reset_password";
/// `{prefix}/{resource}/{id}/rights`: an admin's own rights grid
pub const RIGHTS: &str = "admin_rights";
/// `{prefix}/groupes/rights?groupe={id}` (GET): the group picked in the
/// rights page's selector, redirected to its grid
pub const GROUP_SELECT: &str = "admin_group_select";
/// `{prefix}/{resource}/{id}/groups` (POST): an account's groups
pub const GROUPS: &str = "admin_groups";
/// `{prefix}/groupes/{id}/transfer`: what handing a group over does
/// (GET), then the transfer (POST)
pub const TRANSFER: &str = "admin_transfer";
/// `{prefix}/{resource}/rank` (POST): gives the accounts `ids` a rank
pub const RANK: &str = "admin_rank";
/// `{prefix}/{resource}/bulk` (POST): a grouped action on the checked rows
pub const BULK: &str = "admin_bulk";
/// `{prefix}/history`, filtered by `?resource=&object=&action=&user=`
pub const HISTORY: &str = "admin_history";
/// `{prefix}/history/{id}`: one entry and its changes
pub const HISTORY_ENTRY: &str = "admin_history_entry";
