// This should be transformed (uppercase)
import { observer } from "bonsify";
const Foo = observer(()=><div/>);
// This should NOT be transformed (lowercase)
const bar = ()=><div/>;
// This should be transformed (uppercase)
const Baz = observer(()=><div/>);
export { Foo, bar, Baz };
