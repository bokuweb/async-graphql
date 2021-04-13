use crate::extensions::ResolveInfo;
use crate::parser::types::Field;
use crate::{ContextSelectionSet, OutputType, Positioned, Type, Value};

/// Resolve an list by executing each of the items concurrently.
pub async fn resolve_list<'a, T: OutputType + 'a>(
    ctx: &ContextSelectionSet<'a>,
    field: &Positioned<Field>,
    iter: impl IntoIterator<Item = T>,
    len: Option<usize>,
) -> Value {
    let extensions = &ctx.query_env.extensions;
    if !extensions.is_empty() {
        let mut futures = len.map(Vec::with_capacity).unwrap_or_default();
        for (idx, item) in iter.into_iter().enumerate() {
            futures.push({
                let ctx = ctx.clone();
                async move {
                    let ctx_idx = ctx.with_index(idx);
                    let extensions = &ctx.query_env.extensions;

                    let resolve_info = ResolveInfo {
                        path_node: ctx_idx.path_node.as_ref().unwrap(),
                        parent_type: &Vec::<T>::type_name(),
                        return_type: &T::qualified_type_name(),
                    };
                    let resolve_fut = async { OutputType::resolve(&item, &ctx_idx, field).await };
                    futures_util::pin_mut!(resolve_fut);
                    extensions.resolve(resolve_info, &mut resolve_fut).await
                }
            });
        }
        Value::List(futures_util::future::join_all(futures).await)
    } else {
        let mut futures = len.map(Vec::with_capacity).unwrap_or_default();
        for (idx, item) in iter.into_iter().enumerate() {
            let ctx_idx = ctx.with_index(idx);
            futures.push(async move { OutputType::resolve(&item, &ctx_idx, field).await });
        }
        Value::List(futures_util::future::join_all(futures).await)
    }
}
