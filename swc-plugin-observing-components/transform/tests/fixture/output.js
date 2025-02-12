import { observer } from "bonsify";
export const Home2 = observer(()=>{
    return <div/>;
});
export const Home3 = observer(function() {
    return <div/>;
});
export const Home4 = memo(observer(function() {
    return <div/>;
}));
export const Home5 = memo(observer(()=>{
    return <div/>;
}));
export const Home7 = someWrapper(observer(()=>{
    return <div/>;
}));
export const Home6 = observer(function Home6() {
    return <div/>;
});
const Home8 = observer(function Home8() {
    return <div/>;
});
export default observer(function Home() {
    return <div/>;
});
