// This should be transformed (uppercase)
import { observer } from "bonsify";
const Foo = observer(()=><div/>);
// This should NOT be transformed (lowercase)
const bar = ()=><div/>;
export { Foo, bar };
