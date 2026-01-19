import React, { useEffect, useState } from 'react';
import { api } from '../services';
import type { User } from '../types';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter } from '../components/ui/dialog';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../components/ui/table';
import { Plus, Trash2, RotateCcw, Shield, ShieldOff, Ban, CheckCircle } from 'lucide-react';
import { toast } from 'sonner';

export default function UsersPage() {
    const [users, setUsers] = useState<User[]>([]);
    const [isCreateOpen, setIsCreateOpen] = useState(false);
    const [newUsername, setNewUsername] = useState('');
    const [createdCreds, setCreatedCreds] = useState<{username: string, password: string, title: string} | null>(null);

    useEffect(() => {
        loadUsers();
    }, []);

    const loadUsers = async () => {
        try {
            const data = await api.getUsers();
            setUsers(data);
        } catch (error) {
            toast.error("Failed to load users");
        }
    };

    const handleCreate = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            const result = await api.createUser({ username: newUsername });
            setCreatedCreds({ 
                username: result.user.username, 
                password: result.password,
                title: "User Created!"
            });
            setIsCreateOpen(false);
            setNewUsername('');
            loadUsers();
            toast.success("User created");
        } catch (error) {
            toast.error("Failed to create user");
        }
    };

    const handleDelete = async (id: number) => {
        if (!confirm('Are you sure?')) return;
        try {
            await api.deleteUser(id);
            loadUsers();
            toast.success("User deleted");
        } catch (error) {
            toast.error("Failed to delete user");
        }
    };

    const handleResetPassword = async (id: number) => {
        if (!confirm("Are you sure you want to reset this user's password?")) return;

        try {
            const response = await api.resetUserPassword(id, {});
            loadUsers();
            
            if (response.password) {
                const user = users.find(u => u.id === id);
                if (user) {
                    setCreatedCreds({ 
                        username: user.username, 
                        password: response.password,
                        title: "Password Reset Successful"
                    });
                    toast.success("Password reset successfully");
                }
            }
        } catch (error) {
            toast.error("Failed to reset password");
        }
    };

    const toggleRole = async (user: User) => {
        const newRole = user.role === 'admin' ? 'user' : 'admin';
        if (!confirm(`Change role to ${newRole}?`)) return;
        
        try {
            await api.updateUserRole(user.id, newRole);
            loadUsers();
            toast.success("Role updated");
        } catch (error) {
            toast.error("Failed to update role");
        }
    };

    const toggleStatus = async (user: User) => {
        const action = user.is_disabled ? 'enable' : 'disable';
        if (!confirm(`Are you sure you want to ${action} this user?`)) return;
        
        try {
            await api.updateUserStatus(user.id, !user.is_disabled);
            loadUsers();
            toast.success(`User ${action}d`);
        } catch (error) {
            toast.error("Failed to update status");
        }
    };

    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center">
                <h2 className="text-3xl font-bold tracking-tight">Users</h2>
                <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
                    <DialogTrigger asChild>
                        <Button className="gap-2">
                            <Plus className="h-4 w-4" />
                            Add User
                        </Button>
                    </DialogTrigger>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>Add New User</DialogTitle>
                        </DialogHeader>
                        <form onSubmit={handleCreate} className="space-y-4">
                            <div className="space-y-2">
                                <Label>Username</Label>
                                <Input 
                                    value={newUsername}
                                    onChange={e => setNewUsername(e.target.value)}
                                    required
                                />
                            </div>
                            <DialogFooter>
                                <Button type="submit">Create</Button>
                            </DialogFooter>
                        </form>
                    </DialogContent>
                </Dialog>
            </div>

            {createdCreds && (
                <div className="bg-green-50 border border-green-200 p-4 rounded-md mb-6">
                    <h3 className="font-bold text-green-800">{createdCreds.title}</h3>
                    <p className="text-sm text-green-700 mt-1">
                        Username: <strong>{createdCreds.username}</strong><br/>
                        Temporary Password: <strong className="font-mono bg-white px-1 rounded border">{createdCreds.password}</strong>
                    </p>
                    <p className="text-xs text-green-600 mt-2">
                        Please copy this password immediately. It will not be shown again.
                    </p>
                    <Button variant="outline" size="sm" className="mt-2" onClick={() => setCreatedCreds(null)}>
                        Dismiss
                    </Button>
                </div>
            )}

            <div className="rounded-md border bg-white overflow-hidden">
                <div className="overflow-x-auto">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>Username</TableHead>
                                <TableHead>Role</TableHead>
                                <TableHead>Last Login</TableHead>
                                <TableHead>Status</TableHead>
                                <TableHead className="text-right">Actions</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {users.map(user => (
                                <TableRow key={user.id} className={user.is_disabled ? "bg-gray-50 opacity-75" : ""}>
                                    <TableCell className="font-medium">
                                        {user.username}
                                        {user.is_disabled && <span className="ml-2 text-xs text-red-600 font-bold">(Disabled)</span>}
                                    </TableCell>
                                    <TableCell>
                                        <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${
                                            user.role === 'admin' ? 'bg-purple-100 text-purple-800' : 'bg-gray-100 text-gray-800'
                                        }`}>
                                            {user.role}
                                        </span>
                                    </TableCell>
                                    <TableCell className="whitespace-nowrap">
                                        {user.last_login_at ? new Date(user.last_login_at).toLocaleString() : 'Never'}
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex flex-col gap-1 whitespace-nowrap">
                                            {user.force_password_change && (
                                                <span className="text-xs text-amber-600 font-semibold">
                                                    Password Reset Pending
                                                </span>
                                            )}
                                            {user.is_disabled ? (
                                                <span className="text-xs text-red-600 font-semibold flex items-center gap-1">
                                                    <Ban className="h-3 w-3" /> Disabled
                                                </span>
                                            ) : (
                                                <span className="text-xs text-green-600 font-semibold flex items-center gap-1">
                                                    <CheckCircle className="h-3 w-3" /> Active
                                                </span>
                                            )}
                                        </div>
                                    </TableCell>
                                    <TableCell className="text-right space-x-2 whitespace-nowrap">
                                        <Button size="icon" variant="ghost" title={user.is_disabled ? "Enable User" : "Disable User"} onClick={() => toggleStatus(user)}>
                                            {user.is_disabled ? <CheckCircle className="h-4 w-4 text-green-600" /> : <Ban className="h-4 w-4 text-red-600" />}
                                        </Button>
                                        <Button size="icon" variant="ghost" title="Toggle Role" onClick={() => toggleRole(user)}>
                                            {user.role === 'admin' ? <ShieldOff className="h-4 w-4" /> : <Shield className="h-4 w-4" />}
                                        </Button>
                                        <Button size="icon" variant="ghost" title="Reset Password" onClick={() => handleResetPassword(user.id)}>
                                            <RotateCcw className="h-4 w-4" />
                                        </Button>
                                        <Button size="icon" variant="ghost" className="text-red-500 hover:text-red-600 hover:bg-red-50" onClick={() => handleDelete(user.id)}>
                                            <Trash2 className="h-4 w-4" />
                                        </Button>
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </div>
            </div>
        </div>
    );
}
