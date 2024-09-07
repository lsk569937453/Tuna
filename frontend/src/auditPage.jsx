import { Button, Card, Pagination } from "flowbite-react";
import { Table, Modal } from "flowbite-react";
import { HiOutlinePlus, HiSearch, HiShoppingCart } from "react-icons/hi";
import ReactECharts from 'echarts-for-react';
import Request from "./utils/axiosUtils";
import { useEffect, useState } from "react"; import BatteryGauge from 'react-battery-gauge'
import moment from 'moment';
import 'react-toastify/dist/ReactToastify.css';
import { useNavigate, NavLink } from "react-router-dom";

import { ToastContainer, toast } from 'react-toastify';

const pageSize = 5;

function AuditPage() {

    const [openModal, setOpenModal] = useState(false);
    const [taskId, setTaskId] = useState("");
    //0代表插入，1代表更新
    const [modalType, setModalType] = useState(0);
    const [taskTableData, setTaskTableData] = useState([]);
    const navigate = useNavigate();

    useEffect(() => {
        getTaskList();
    }, []);


    const getTaskList = () => {
        Request.get("/api/auditTask").then((res) => {
            console.log(res);
            const mesArray = res.data.message.map(
                ({
                    id: id,
                    task_id: task_id,
                    status: status,

                    timestamp: timestamp

                }) => {
                    return {
                        id,
                        task_id,
                        status,
                        timestamp

                    };
                }
            );
            setTaskTableData(mesArray);
        });
    };

    const addAuditTask = () => {
        Request.post("/api/auditTask", {
            "task_id": Number(taskId),

        }).then((res) => {

            console.log(res);
            if (res.data.resCode != 0) {
                // this.$message.error('添加错误:' + res.data.message);
                toast.error("添加任务出错!", {
                    position: "top-center"
                });
            } else {

                toast.info("添加任务成功!", {
                    position: "top-center"
                });
                window.location.reload();
            }
        })
            .catch(err => {
                console.log(err)
            });
    }
    const deleteTask = (id) => {
        Request.delete("/api/auditTask/" + id).then((res) => {
            console.log(res);
            if (res.data.resCode != 0) {
                // this.$message.error('添加错误:' + res.data.message);
                toast.error("删除任务出错!", {
                    position: "top-center"
                });
            } else {
            }
            window.location.reload();

        })
    }
    const executeAuditTask = (id) => {
        let jsonBody = {
            "id": id
        };
        Request.post("/api/auditTask/execute", jsonBody).then((res) => {
            console.log(res);
            if (res.data.resCode != 0) {
                // this.$message.error('添加错误:' + res.data.message);
                toast.error("执行任务出错!", {
                    position: "top-center"
                });
            } else {
            }
            window.location.reload();

        })
    }
    const statusText = (status) => {
        if (status == 1) {
            return "运行中";
        } else if (status == 2) {
            return "执行完成";
        } else if (status == 0) {
            return "未开始";
        }
    }
    const handleClick = (auditTaskId) => {
        navigate('/auditResultPage?auditTaskId=' + auditTaskId);
    };


    return (
        <div className="flex flex-col">

            <div className="p-4 flex-col">
                <div className="mb-4 flex justify-center">

                    <Button onClick={() => setOpenModal(true)}>添加稽核任务</Button>
                    <ToastContainer />

                    <Modal dismissible show={openModal} onClose={() => setOpenModal(false)} >
                        <div className="flex flex-col items-center gap-4 p-5 ">
                            <div className="flex items-center w-full">
                                <span className="mr-2 basis-1/3 text-right	">任务Id:</span>
                                <input type="text" className="basis-1/3 bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block  p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="数据源名称" required
                                    onChange={(e) => setTaskId(e.target.value)}
                                    value={taskId}
                                />
                            </div>


                            <div className="flex items-center  w-full">
                                <div className="mr-2  basis-1/3">
                                </div>
                                {modalType == 0 &&
                                    <Button className="basis-1/3" onClick={addAuditTask}>添加</Button>}
                                {modalType == 1 &&
                                    <Button className="basis-1/3" onClick={confirmEditFurnace}>更新</Button>}
                            </div>
                        </div>

                    </Modal>
                </div>
                <Table>
                    <Table.Head>
                        <Table.HeadCell className="font-bold text-center text-xl">id</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">任务id</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">状态</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">时间</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">操作</Table.HeadCell>

                    </Table.Head>
                    <Table.Body className="divide-y">
                        {taskTableData.map((row, index) => (
                            <Table.Row className="bg-white dark:border-gray-700 dark:bg-gray-800" key={index}>
                                <Table.Cell className="whitespace-nowrap font-medium text-gray-900 dark:text-white text-center">
                                    {row.id}
                                </Table.Cell>
                                <Table.Cell className="text-center">  {row.task_id}</Table.Cell>
                                <Table.Cell className="text-center">  {statusText(row.status)}</Table.Cell>
                                <Table.Cell className="text-center">  {row.timestamp}</Table.Cell>

                                <Table.Cell className="text-center">
                                    <div className="flex flex-row space-x-4 justify-center">
                                        <a href="#" className="font-medium text-cyan-600 hover:underline dark:text-cyan-500"
                                            onClick={() => deleteTask(row.id)}>
                                            删除
                                        </a>
                                        <a href="#" className="font-medium text-cyan-600 hover:underline dark:text-cyan-500"
                                            onClick={() => executeAuditTask(row.id)}
                                        >
                                            执行
                                        </a>
                                        <a href="#" className="font-medium text-cyan-600 hover:underline dark:text-cyan-500"
                                            onClick={() => handleClick(row.id)}
                                        >
                                            查看
                                        </a>
                                    </div>
                                </Table.Cell>

                            </Table.Row>
                        ))}

                    </Table.Body>
                </Table>

            </div>
        </div >
    );
}

export default AuditPage;