import { CssBaseline } from '@mui/material'
import NavBar from '@/components/Navbar'
import LoginForm from '@/components/auth/LoginForm'

import {
  AuthProvider,
  AuthIsNotSignedIn, 
  AuthIsSignedIn,
} from "@/contexts/AuthContext";

import {
  Navigate,
  Route,
  BrowserRouter as Router,
  Routes,
} from "react-router-dom";
import Logout from './components/auth/Logout';
import UserHome from './components/UserHome';
import AboutApp from './components/About';

function App() {
  return (
    // Everything has access to authentication context
    // Page Shell/Layout is outside the router
    <CssBaseline>
    <AuthProvider>      
        <NavBar/>
        <AuthIsSignedIn>
          <Router>
            <Routes>        
              <Route path={"/logout"} element={<Logout/>} />
              <Route path={"/home"}   element={<UserHome/>} />
              <Route path={"/*"}      element={<Navigate replace to={"/home"}/>} />
            </Routes>
          </Router>
        </AuthIsSignedIn>

        <AuthIsNotSignedIn>
          <Router>
            <Routes>
              <Route path={"/login"} element={<LoginForm postLoginRoute="/home"/>} />
              <Route path={"/about"} element={<AboutApp/>} />
              <Route path={"/*"}     element={<Navigate replace to={"/login"}/>} />
            </Routes>
          </Router>          
        </AuthIsNotSignedIn>

    </AuthProvider>
    </CssBaseline>
  )
}

export default App
