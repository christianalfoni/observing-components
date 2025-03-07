import { observer } from "bonsify";
const FOO = "Foo";
const bar = "bar";
const components = {
    [FOO]: observer(()=><div/>),
    // This should NOT be transformed (lowercase computed key)
    [bar]: ()=><div/>,
    [Page.Foo]: observer(()=><div/>)
};
export default components;
