// sourced from 
// - https://gist.github.com/herrera-ignacio/bb9d0cc089c798cb240b85aa4e639b62
// - https://medium.com/@remind.stephen.to.do.sth/hands-on-guide-to-secure-react-routes-with-authentication-context-971f37ede990

import React, { FunctionComponent, useContext, useEffect, useState } from "react";
import { AuthenticateRequest} from "../api/auth";

export enum AuthStatus {
    Loading,
    SignedIn,
    SignedOut
}

//--------------------------------------------------
type TCallbackFunction = () => void;
type TCallbackFunctionVariadic = (...args: unknown[]) => void;

export interface IAuth {
    authStatus? : AuthStatus;
    signIn? : TCallbackFunction;
    signOut? : TCallbackFunction;
}

const defaultState: IAuth = {
    authStatus: AuthStatus.Loading
};

export const AuthContext = React.createContext(defaultState);

// Rename ?
// These return children (filter) depending on logged in or not.
type Props = {
    children?: React.ReactNode;
};

export const AuthIsSignedIn = ({children}: Props) => {
    const {authStatus} : IAuth = useContext(AuthContext);
    return (
        <> 
            {console.log(`In AuthIsSignedIn with authStatus=${authStatus}`)}
            {authStatus === AuthStatus.SignedIn ? children : null } 
        </>
    );
};


export const AuthIsNotSignedIn = ({children}: Props) => {
    const {authStatus} : IAuth = useContext(AuthContext);
    return (
        <> 
            {console.log(`In AuthIsNotSignedIn with authStatus=${authStatus}`)}
            {authStatus === AuthStatus.SignedOut ? children : null } 
        </>
    );
};

//--------------------------------------------------------------------------
export const AuthProvider: FunctionComponent<Props> = ({children} : Props) => {
    const [authStatus, setAuthStatus] = useState(AuthStatus.Loading);

    // Effect when setAuthStatus or authStatus changes.
    // Not sure what `setAuthStatus` is also included. 
    // How will this method change ?
    //
    // don't see how putting [authStatus, setAuthStatus] in dependencies will work. 
    // Won't it be an infinite loop. Making it [] will cause it to trigger each render.
    // but I have no idea what the context use has to do with rendering. Yet!
    useEffect( () => {
        async function getAuthenicate() {            
            const authResult = await AuthenticateRequest();
            if (authResult != "" && authResult) {
                setAuthStatus(AuthStatus.SignedIn);
            } else {
                setAuthStatus(AuthStatus.SignedOut);    
            }
        }

        getAuthenicate().then( 
            () => {}
        ).catch(
            (error) => {
                setAuthStatus(AuthStatus.SignedOut)
            }
        )
    },     
    [authStatus, setAuthStatus]);

    function _onSignOut() {        
        setAuthStatus(AuthStatus.SignedOut);
    }

    function _onSignIn() {
        setAuthStatus(AuthStatus.SignedIn);
    }

    const state : IAuth = {
        authStatus: authStatus,
        signIn    : _onSignIn,
        signOut   : _onSignOut,
    };

    // By now, we should be in or the other
    // Why not throw instead of null ? - vj
    if (authStatus === AuthStatus.Loading)
        return null;

    return (
    <AuthContext.Provider value={state}>
        {children}
    </AuthContext.Provider>);
}
