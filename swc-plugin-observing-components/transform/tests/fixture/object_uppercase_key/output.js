import { observer } from "bonsify";
const components = {
    // This should be transformed (uppercase key)
    FOO: observer(()=><div/>),
    // This should NOT be transformed (lowercase key)
    bar: ()=><div/>
};
export default components;
