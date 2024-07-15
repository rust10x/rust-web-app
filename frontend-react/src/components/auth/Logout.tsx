import { Button } from "@mui/material";
import { FunctionComponent } from "react";
import { LogoutRequest } from "../../api/auth";

interface LogoutProps {
    
}
 
const Logout: FunctionComponent<LogoutProps> = () => {
    function handleLogout() {
        console.log("Logging out");
        LogoutRequest().then();
    }

    return (
        <>
        <h1> Log Out </h1>
        <Button onClick={handleLogout}> Logout </Button>
        </>
    );
}
 
export default Logout;