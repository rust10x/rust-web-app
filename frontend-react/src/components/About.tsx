import { Link } from "@mui/material"
import { FunctionComponent } from "react";

interface AboutAppProps {
    
}
 
const AboutApp: FunctionComponent<AboutAppProps> = () => {
    return (
    <>
        <h1>About Rust10x Web App</h1>
        <p>
            See <Link href="https://rust10x.com/web-app">rust10x project</Link> and <Link href="https://github.com/rust10x/rust-web-app">github</Link>
        </p>
    </>
);
}
 
export default AboutApp;