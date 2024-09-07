import { Button, Card, Pagination } from "flowbite-react";
import { Table, Modal } from "flowbite-react";
import { HiOutlinePlus, HiSearch, HiShoppingCart } from "react-icons/hi";
import ReactECharts from 'echarts-for-react';
import Request from "./utils/axiosUtils";
import { useEffect, useState } from "react"; import BatteryGauge from 'react-battery-gauge'
import moment from 'moment';
import 'react-toastify/dist/ReactToastify.css';
import { useSearchParams } from 'react-router-dom';

import { ToastContainer, toast } from 'react-toastify';

const pageSize = 5;

function AuditResultPage() {

    const [openModal, setOpenModal] = useState(false);
    const [datasourceName, setDatasourceName] = useState("");
    const [datasourceUrl, setDatasourceUrl] = useState("");
    //0代表插入，1代表更新
    const [modalType, setModalType] = useState(0);
    const [taskTableData, setTaskTableData] = useState([]);
    const [searchParams] = useSearchParams();
    const auditTaskId = searchParams.get('auditTaskId');
    useEffect(() => {
        if (auditTaskId)
            getTaskList();
    }, []);


    const getTaskList = () => {
        Request.get("/api/auditTaskResult/" + auditTaskId).then((res) => {
            console.log(res);
            const mesArray = res.data.message.map(
                ({
                    id: id,
                    audit_task_id: audit_task_id,
                    execution_id: execution_id,
                    primary_id: primary_id,
                    left_compare: left_compare,
                    right_compare: right_compare,
                    is_same: is_same,
                    timestamp: timestamp

                }) => {
                    return {
                        id,
                        audit_task_id,
                        execution_id,
                        primary_id,
                        left_compare,
                        right_compare,
                        is_same,
                        timestamp

                    };
                }
            );
            setTaskTableData(mesArray);
        });
    };

    const addDatasource = () => {
        Request.post("/api/datasource", {
            "datasource_name": datasourceName,
            "datasource_url": datasourceUrl,

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
        if (status == 0) {
            return "相同";
        } else {
            return "不相同";
        }
    }
    return (
        <div className="flex flex-col">

            <div className="p-4 flex-col">
                <div className="mb-4 flex justify-center">

                    <ToastContainer />


                </div>
                <Table>
                    <Table.Head>
                        <Table.HeadCell className="font-bold text-center text-xl">稽核任务id</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">批次id</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">主键</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">主表</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">迁移表</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">是否一致</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">时间</Table.HeadCell>

                    </Table.Head>
                    <Table.Body className="divide-y">
                        {taskTableData.map((row, index) => (
                            <Table.Row className="bg-white dark:border-gray-700 dark:bg-gray-800" key={index}>

                                <Table.Cell className="text-center">  {row.audit_task_id}</Table.Cell>
                                <Table.Cell className="text-center">  {row.execution_id}</Table.Cell>
                                <Table.Cell className="text-center">  {row.primary_id}</Table.Cell>
                                <Table.Cell className="text-center">  {row.left_compare}</Table.Cell>
                                <Table.Cell className="text-center">  {row.right_compare}</Table.Cell>

                                <Table.Cell className="text-center">  {statusText(row.is_same)}</Table.Cell>
                                <Table.Cell className="text-center">  {row.timestamp}</Table.Cell>

                                {/* <Table.Cell className="text-center">
                                    <div className="flex flex-row space-x-4">
                                        <a href="#" className="font-medium text-cyan-600 hover:underline dark:text-cyan-500"
                                            onClick={() => deleteTask(row.id)}>
                                            删除
                                        </a>
                                        <a href="#" className="font-medium text-cyan-600 hover:underline dark:text-cyan-500"
                                            onClick={() => executeAuditTask(row.id)}
                                        >
                                            执行
                                        </a>
                                    </div>
                                </Table.Cell> */}

                            </Table.Row>
                        ))}

                    </Table.Body>
                </Table>

            </div>
        </div >
    );
}

export default AuditResultPage;