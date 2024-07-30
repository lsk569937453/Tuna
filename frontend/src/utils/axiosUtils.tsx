import axios from 'axios'; // 引入axios
import { ToastContainer, toast } from 'react-toastify';


var Request = axios.create(
    {

        timeout: 1000000,
        headers: { 'X-Custom-Header': 'foobar', 'userAgent': navigator.userAgent }
    }
);

Request.interceptors.response.use((config) => {
    // config.headers['request-startTime'] = process.hrtime()
    // console.log("send req:" + new Date())
    return config
}, (error) => {
    const { response } = error;
    errorHandle(response.status, response.data.msg);

    return Promise.reject(error);
}

)
const errorHandle = (status: any, other: any) => {
    console.log(status, other);
    switch (status) {
        case 403:
            toast.error("登录过期，请重新登录!", {
                position: "top-center"
            });
            // 清除token
            // localStorage.removeItem('token');
            // store.commit('loginSuccess', null);
            setTimeout(() => {
                // toLogin();
            }, 1000);
            break;
        case 404:
            toast.error("网络请求不存在!", {
                position: "top-center"
            });
            break;
        case 400:
            toast.error("请求参数错误!", {
                position: "top-center"
            });
            break;
        default:
            toast.error(other, {
                position: "top-center"
            });
    }
}

export default Request
