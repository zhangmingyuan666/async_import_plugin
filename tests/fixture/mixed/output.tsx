const c = ()=>{
    return import(/* webpackChunkName: "32-async-component" */ "@/components/async-component.vue").then((res)=>res);
};
