import React, { useState } from 'react';
import { api } from '../services';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { Card, CardHeader, CardTitle, CardContent, CardFooter } from '../components/ui/card';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Button } from '../components/ui/button';
import { Alert, AlertDescription } from '../components/ui/alert';
import { toast } from 'sonner';

export default function ChangePasswordPage() {
    const navigate = useNavigate();
    const { user } = useAuth();
    const [oldPassword, setOldPassword] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [error, setError] = useState('');
    const [isLoading, setIsLoading] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');

        if (newPassword !== confirmPassword) {
            setError("New passwords do not match");
            return;
        }

        setIsLoading(true);

        try {
            await api.changePassword({ old_password: oldPassword, new_password: newPassword });
            toast.success("Password changed successfully");
            navigate('/');
        } catch (err: any) {
            setError(err.response?.data?.error || "Failed to change password");
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="flex items-center justify-center min-h-[80vh]">
            <Card className="w-[400px]">
                <CardHeader>
                    <CardTitle>Change Password</CardTitle>
                </CardHeader>
                <form onSubmit={handleSubmit}>
                    <CardContent className="space-y-4">
                        {user?.force_password_change && (
                            <Alert>
                                <AlertDescription>
                                    You must change your password to continue.
                                </AlertDescription>
                            </Alert>
                        )}
                        
                        {error && (
                            <Alert variant="destructive">
                                <AlertDescription>{error}</AlertDescription>
                            </Alert>
                        )}
                        
                        <div className="space-y-2">
                            <Label htmlFor="old-password">Current Password</Label>
                            <Input 
                                id="old-password" 
                                type="password"
                                value={oldPassword} 
                                onChange={(e) => setOldPassword(e.target.value)}
                                required 
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="new-password">New Password</Label>
                            <Input 
                                id="new-password" 
                                type="password"
                                value={newPassword} 
                                onChange={(e) => setNewPassword(e.target.value)}
                                required 
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="confirm-password">Confirm New Password</Label>
                            <Input 
                                id="confirm-password" 
                                type="password"
                                value={confirmPassword} 
                                onChange={(e) => setConfirmPassword(e.target.value)}
                                required 
                            />
                        </div>
                    </CardContent>
                    <CardFooter>
                        <Button type="submit" className="w-full" disabled={isLoading}>
                            {isLoading ? 'Updating...' : 'Update Password'}
                        </Button>
                    </CardFooter>
                </form>
            </Card>
        </div>
    );
}
