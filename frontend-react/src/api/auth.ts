import { IUser} from "../model/iuser.interface";

export async function AuthenticateRequest(): Promise<IUser|null> {
    const signedInUser = localStorage.getItem("username");
    if ( signedInUser ) {
        return {username: signedInUser, first: "John", last: "Doe"};
    } else {
        return null;
    }
}

export async function LoginRequest(username: string, pwd: string, rememberMe: boolean) : Promise<IUser|null> {
    
    // Mock functionlaity
    if ( username === "user" && pwd === "pwd") {
        localStorage.setItem("username", username);
        return AuthenticateRequest();
    } else
    {
        localStorage.removeItem("username");
        throw new Error("Login failed");
    }
}

export async function LogoutRequest() {
    // Mock impl.
    localStorage.removeItem("username");
}
