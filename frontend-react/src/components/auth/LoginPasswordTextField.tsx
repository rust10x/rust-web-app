import { Visibility, VisibilityOff } from "@mui/icons-material";
import { IconButton, InputAdornment, TextField } from "@mui/material";
import { FunctionComponent, useState } from "react";

interface LoginFormPasswordTextFieldProps {
    handleChange : React.ChangeEventHandler<HTMLInputElement>,
    showPassword : boolean,
    errorState   : boolean,
    errorMsg     : string,
}

//------------------------------------------------------------------------------------------
// changeHandler is via the textField's onChange handler and the event is passed as-is.
// The `id` will be `password`. Use it like this..
//
// function handleChange(e) => {
//    // id will be `password`
//    const {id, value} = e.target;
// }
//
// <Container component="form" ....>
//    ...
//    <LoginFormPasswordTextField handleChange={changeHandler} showPassword=false />
// </Container>
//------------------------------------------------------------------------------------------
const LoginFormPasswordTextField: FunctionComponent<LoginFormPasswordTextFieldProps> = (
    props: LoginFormPasswordTextFieldProps) => {

    const [showPassword, setShowPassword] = useState(props.showPassword);
    const handleClickShowPassword = () => setShowPassword(!showPassword);
    const handleMouseDownPassword = () => setShowPassword(!showPassword);

    return (  
        <TextField 
            label='Password' placeholder='Enter password' 
            type={showPassword ? "text" : "password"}                // <-- this makes showPassword changes to re-render this field
            id="password"                        
            variant="outlined" fullWidth required
            error = {props.errorState ? true : false}               // <-- login errors
            helperText = {props.errorState ? props.errorMsg : null} // <-- login errors
            InputProps={{                                           // <-- This is where the toggle button is added.
                endAdornment: (
                    <InputAdornment position="end">
                    <IconButton
                        aria-label="toggle password visibility"
                        onClick={handleClickShowPassword}
                        onMouseDown={handleMouseDownPassword}
                    >
                        {showPassword ? <Visibility /> : <VisibilityOff />}
                    </IconButton>
                    </InputAdornment>
                )
                }}
                
            onChange={props.handleChange}/>
    );
}
 
export default LoginFormPasswordTextField;