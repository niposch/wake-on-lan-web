import React, { useState } from 'react';
import { useAuth } from '../context/AuthContext';
import { useNavigate } from 'react-router-dom';
import { Card, CardHeader, CardTitle, CardContent, CardFooter, CardDescription } from '../components/ui/card';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Button } from '../components/ui/button';
import { Alert, AlertDescription } from '../components/ui/alert';
import { Server } from 'lucide-react';

export default function LoginPage() {
    const { login } = useAuth();
    const navigate = useNavigate();
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [rememberMe, setRememberMe] = useState(false);
    const [error, setError] = useState('');
    const [isLoading, setIsLoading] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        setIsLoading(true);

        try {
            await login({ username, password, remember_me: rememberMe });
            navigate('/');
        } catch (err: any) {
            console.error(err);
            const message = err.response?.data?.error || 'Invalid credentials';
            setError(message);
        } finally {
            setIsLoading(false);
        }
    };

    const isDemo = import.meta.env.VITE_USE_MOCK_API === 'true';

    const fillCredentials = (role: 'admin' | 'user') => {
        setUsername(role);
        setPassword(role);
    };

    return (
        <div className="flex flex-col items-center justify-center min-h-screen bg-gray-50 p-4">
            <div className="mb-8 flex flex-col items-center gap-2">
                <div className="bg-primary/10 p-3 rounded-full">
                    <Server className="h-8 w-8 text-primary" />
                </div>
                <h1 className="text-2xl font-bold tracking-tight">WoL Manager</h1>
                <p className="text-sm text-muted-foreground">Wake your devices from anywhere</p>
            </div>
            
            <Card className="w-full max-w-sm shadow-md">
                <CardHeader>
                    <CardTitle>Welcome back</CardTitle>
                    <CardDescription>Enter your credentials to sign in</CardDescription>
                </CardHeader>
                <form onSubmit={handleSubmit}>
                    <CardContent className="space-y-4">
                        {error && (
                            <Alert variant="destructive">
                                <AlertDescription>{error}</AlertDescription>
                            </Alert>
                        )}
                        <div className="space-y-2">
                            <Label htmlFor="username">Username</Label>
                            <Input 
                                id="username" 
                                value={username} 
                                onChange={(e) => setUsername(e.target.value)}
                                required 
                                placeholder="Enter your username"
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="password">Password</Label>
                            <Input 
                                id="password" 
                                type="password" 
                                value={password} 
                                onChange={(e) => setPassword(e.target.value)}
                                required 
                                placeholder="Enter your password"
                            />
                        </div>
                        <div className="flex items-center space-x-2 py-2">
                            <input
                                type="checkbox"
                                id="remember_me"
                                checked={rememberMe}
                                onChange={(e) => setRememberMe(e.target.checked)}
                                className="h-4 w-4 rounded border-gray-300 accent-primary cursor-pointer transition-all hover:scale-110"
                            />
                            <Label 
                                htmlFor="remember_me" 
                                className="text-sm font-medium leading-none cursor-pointer text-gray-600 hover:text-gray-900 transition-colors"
                            >
                                Remember me for 30 days
                            </Label>
                        </div>
                    </CardContent>
                    <CardFooter className="pt-2">
                        <Button type="submit" className="w-full h-11 shadow-sm" disabled={isLoading}>
                            {isLoading ? 'Logging in...' : 'Login'}
                        </Button>
                    </CardFooter>
                </form>
            </Card>

            {isDemo && (
                <div className="mt-8 text-center space-y-3">
                    <p className="text-sm text-muted-foreground font-medium">Demo Mode Quick Login</p>
                    <div className="flex gap-3">
                        <Button variant="outline" size="sm" onClick={() => fillCredentials('admin')}>
                            Auto-fill Admin
                        </Button>
                        <Button variant="outline" size="sm" onClick={() => fillCredentials('user')}>
                            Auto-fill User
                        </Button>
                    </div>
                </div>
            )}
        </div>
    );
}
