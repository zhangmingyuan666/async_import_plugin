# TARGET
tramsform s1sAsyncImport to import function with magic comments and
```js
const c = s1sAsyncImport("@/components/async-component.vue")
```
function to 
```rust
const c = ()=>{
    return import(/* webpackChunkName: chunkIndex */"@/components/async-component.vue").then((res)=>res);
};
```

# HOW TO USE
how to build
```shell
cargo build
```
how to test
```shell
cargo build
cargo test --package s1s_async_import_plugin --test fixture --  --show-output 
```

