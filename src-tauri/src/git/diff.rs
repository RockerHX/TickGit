use crate::error::AppResult;

use super::{
    command::{git_text, git_trimmed},
    repository::resolve_repository_path,
};

// Git 约定的空树对象。初始提交没有 parent 时，使用它与目标提交做 diff，
// 才能和普通提交一样统一走 diff 参数（例如 -w 忽略空白）逻辑。
const EMPTY_TREE_HASH: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

pub fn get_commit_file_diff(
    repo_path: &str,
    hash: &str,
    file_path: &str,
    previous_path: Option<&str>,
    ignore_whitespace: bool,
) -> AppResult<String> {
    let repo_path = resolve_repository_path(repo_path)?;
    let parents = git_trimmed(&repo_path, &["show", "-s", "--format=%P", hash])?;
    let whitespace_arg = ignore_whitespace.then_some("-w");
    let mut pathspecs = Vec::new();

    if let Some(previous_path) = previous_path
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != file_path)
    {
        pathspecs.push(previous_path);
    }

    pathspecs.push(file_path);

    if parents.trim().is_empty() {
        let mut args = vec!["show"];
        if let Some(arg) = whitespace_arg {
            // 初始提交如果继续走 git show -w，不同 Git 版本下空白过滤语义不够稳定；
            // 这里改成 empty-tree -> commit 的 diff，和普通提交保持一致。
            args = vec!["diff"];
            args.push(arg);
            args.extend(["--find-renames", EMPTY_TREE_HASH, hash, "--"]);
            args.extend(pathspecs.iter().copied());
            return git_text(&repo_path, &args);
        }
        args.extend(["--find-renames", "--format=", hash, "--"]);
        args.extend(pathspecs.iter().copied());
        return git_text(&repo_path, &args);
    }

    let parent_ref = format!("{hash}^");
    let mut args = vec!["diff"];
    if let Some(arg) = whitespace_arg {
        args.push(arg);
    }
    args.extend(["--find-renames", parent_ref.as_str(), hash, "--"]);
    args.extend(pathspecs.iter().copied());
    git_text(&repo_path, &args)
}
