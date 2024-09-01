import { DarkThemeToggle, Navbar } from "flowbite-react";
import { Sidebar, Button, Dropdown } from "flowbite-react";
import { HiArrowSmRight, HiChartPie, HiInbox, HiShoppingBag, HiChip, HiUser, HiViewBoards, HiOutlineLogout, HiLogout } from "react-icons/hi";
import Dashboard from "./dashboard";
import DatasourcePage from "./datasourcePage";
import TaskPage from "./taskPage";
import { Routes, Route, Outlet, Link, Navigate } from 'react-router-dom';
import { useState, useEffect } from "react"
import moment from 'moment';
import { CookiesProvider, useCookies } from 'react-cookie'
import { useNavigate, NavLink } from "react-router-dom";
import 'moment/dist/locale/zh-cn';
import AuditPage from "./auditPage";
import AuditResultPage from "./auditResultPage";
const customTheme = {
    root: {
        base: 'inherit',
        inner: 'inherit',

    },
};
moment.locale('zh-cn');

function HomePage() {
    const [currentDate, setCurrentDate] = useState("");
    const [cookies, setCookie, removeCookie] = useCookies(['guestAuthority'])
    const navigate = useNavigate();

    // useEffect(() => {
    //     console.log("cookie is" + cookies);
    //     if (cookies.guestAuthority === undefined) {
    //         navigate("/login");
    //     }
    // }, []);

    useEffect(() => {
        let currentdate = moment().format('dddd YYYY/MM/DD, HH:mm:ss');
        setCurrentDate(currentdate);
        const interval = setInterval(() => {
            console.log(moment.locale());
            let currentdate = moment().format('dddd YYYY/MM/DD, HH:mm:ss');
            setCurrentDate(currentdate);

        }, 1000);
        return () => clearInterval(interval);
    }, []);


    const clearCookie = () => {
        removeCookie("guestAuthority");
        navigate("/login");

    }

    return (
        <main className="flex flex-col h-screen overflow-hidden	">

            <div className="max-w-full flex h-full">
                <div className="basis-1/12 h-full bg-sky-200		">
                    <Sidebar theme={customTheme}>
                        <Sidebar.Items>
                            <Sidebar.ItemGroup >
                                <Link to="/dashboard" >
                                    <Sidebar.Item icon={HiChartPie} >
                                        表盘
                                    </Sidebar.Item>
                                </Link>
                                <Link to="/taskPage" >

                                    <Sidebar.Item icon={HiViewBoards} labelColor="dark" >
                                        任务管理
                                    </Sidebar.Item>
                                </Link>

                                <Link to="/datasourcePage" >
                                    <Sidebar.Item icon={HiChip} >
                                        数据源管理
                                    </Sidebar.Item>
                                </Link>
                                <Link to="/auditPage" >
                                    <Sidebar.Item icon={HiChip} >
                                        稽核管理
                                    </Sidebar.Item>
                                </Link>

                                <Link to="/auditResultPage" >
                                    <Sidebar.Item icon={HiChip} >
                                        稽核结果
                                    </Sidebar.Item>
                                </Link>

                            </Sidebar.ItemGroup>
                        </Sidebar.Items>
                    </Sidebar>
                </div>
                <div className="basis-11/12 h-full">
                    <Routes>
                        <Route path="/" >
                            <Route
                                index
                                element={<Navigate to="/dashboard" />}
                            />
                            <Route path="dashboard" element={<Dashboard />} />
                            <Route path="datasourcePage" element={<DatasourcePage />} />

                            <Route path="taskPage" element={<TaskPage />} />
                            <Route path="auditPage" element={<AuditPage />} />
                            <Route path="auditResultPage" element={<AuditResultPage />} />


                        </Route>
                    </Routes>
                </div>
            </div>

        </main>
    );
}

export default HomePage;