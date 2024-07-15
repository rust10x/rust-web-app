import { DirectionsRun } from '@mui/icons-material';
import { AppBar, Button, IconButton, Stack, Toolbar, Typography } from '@mui/material';
import {FunctionComponent} from 'react';

interface NavBarProps {
    
}
 
const NavBar: FunctionComponent<NavBarProps> = () => {
    return (  
        <AppBar position='static'>
            <Toolbar>                
                <IconButton size='large' edge='start' color='inherit' aria-label='logo'>
                    <DirectionsRun/>
                </IconButton>

                <Typography variant='h6' component='div' sx={{flexGrow:1}}>
                    Rust10x Web App
                </Typography>
                
                <Stack direction='row' spacing={2}>
                    <Button color='inherit'>About</Button>
                    <Button color='inherit'>Login</Button>
                </Stack>
            </Toolbar>
        </AppBar>
    );
}
 
export default NavBar;