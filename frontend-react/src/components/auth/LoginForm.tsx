// Copied from gist https://gist.github.com/vaish567/861e88d0e7f13cb00ef88767cd2f8d0f
// and updated for latest materialUI
import {Grid, Paper, Avatar, TextField, FormControlLabel, Checkbox, Button, Typography, Link, Container} from '@mui/material'
import {FunctionComponent, useContext, useState} from 'react'
import LockOutlinedIcon from '@mui/icons-material/LockOutlined'
import { LoginRequest } from '../../api/auth'
import LoginFormPasswordTextField from './LoginPasswordTextField'
import { AuthContext } from '../../contexts/AuthContext'
import { useNavigate } from 'react-router-dom'

interface LoginFormProps {
    postLoginRoute : string
}
 
const LoginForm: FunctionComponent<LoginFormProps> = ({postLoginRoute} : LoginFormProps) => {
    const paperStyle={padding :20,height:'50vh',width:280, margin:"20px auto"}
    const avatarStyle={backgroundColor:'#1bbd7e'}
    const btnstyle={margin:'8px 0'}

    const authContext = useContext(AuthContext);
    const navigate    = useNavigate();

    // error display handling
    const [loginError, setLoginError] = useState(false);
    const LOGIN_ERR_MSG = 'Wrong credentials!';

    // Credential handling
    const [credentials, setCredentials] = useState({
        username: "",
        password: "",
        rememberMe: false,
    });

    const handleChange = (e:React.ChangeEvent<HTMLInputElement>) => {
        const {id, value} = e.target;
        // Note that this needs the "id" prop specified on the controls
        setCredentials((prevCredentials) => ({
            ...prevCredentials,
            [id]: value,
        }))        

        // clear login error right here
        setLoginError(false);
    }    

    const handleSubmit = (e:React.ChangeEvent<HTMLInputElement>) => {
        // simply prevents default submit action from base classes.
        e.preventDefault();

        console.log(`Submitting login info: l=${credentials.username}, p=${credentials.password}, r=${credentials.rememberMe}`);

        // Send to the API
        LoginRequest(credentials.username, credentials.password, credentials.rememberMe).
            then( (response) => {
                // If successful, set the auth status in the context
                if(response) {
                    console.log(`Successful login attempt: username=${response.username}`);
                    authContext.signIn?.();
                    console.log(`Navigating to ${postLoginRoute} on successful login.`)                    
                    navigate(postLoginRoute);
                } else{
                    console.log(`login attempt but empty result. Treating as failure`);
                    authContext.signOut?.();
                    setLoginError(true);
                }
            }
            ).catch((l_err) => {
                console.log(`Error during login: ${l_err}, SigningOut`);
                authContext.signOut?.();
                setLoginError(true);
            });        
    }

    return (  
        <Container component="form" onSubmit={handleSubmit} id="loginform">
            <Grid>
                <Paper elevation={10} style={paperStyle}>
                    <Grid alignItems='center'>
                        <Avatar style={avatarStyle}><LockOutlinedIcon/></Avatar>
                        <h2>Sign In</h2>
                    </Grid>
                    <TextField 
                        label='Username' placeholder='Enter username' 
                        id="username" variant="outlined" fullWidth required
                        error = {loginError ? true : false}
                        helperText = {loginError ? LOGIN_ERR_MSG : null}
                        onChange={handleChange}                    
                        />

                    <LoginFormPasswordTextField 
                        handleChange={handleChange} 
                        showPassword={false} 
                        errorState={loginError}
                        errorMsg={LOGIN_ERR_MSG}
                        />

                    <FormControlLabel
                        control={
                        <Checkbox
                            name="checked"
                            color="primary"
                            id="rememberMe"
                            onChange={handleChange}
                        />
                        }
                        label="Remember me"
                    />

                    <Button 
                        type='submit' color='primary' variant="contained" style={btnstyle} fullWidth>                  
                        Sign in
                    </Button>

                    <Typography >
                        <Link href="#" >
                            Forgot password ?
                    </Link>
                    </Typography>
                    <Typography > Don't have an account ?
                        <Link href="#" >
                            Sign Up 
                    </Link>
                    </Typography>
                </Paper>
            </Grid>    
        </Container>
    );
}
 
export default LoginForm;